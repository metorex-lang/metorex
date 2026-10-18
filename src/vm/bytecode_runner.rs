//! Running a compiled unit: one loop dispatching each instruction by
//! `match`, over a register file the unit's size. An instruction that needs
//! the interpreter, to call a method, read a constant, or raise, calls a
//! `VirtualMachine` function.

use crate::ast::{BinaryOp, Expression, Statement, UnaryOp};
use crate::bytecode::instruction::{
    BINARY_OPERATORS, Instruction, Operand, RANGE_EXCLUSIVE, RANGE_LITERAL_ENDS,
};
use crate::bytecode::unit::{
    CallSite, Definition, HandlerKind, HashEntry, HashShape, OpenAtHandler, Piece, Receiver,
    StartKind, StatementStart, Target, Unit,
};
use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::shape::SlotCache;
use crate::object::{Object, View};
use crate::vm::errors::loop_control_error;
use crate::vm::eval::{CallRoute, SuperArguments};
use crate::vm::expression::{HashLiteral, Interpolation, WrittenPair};
use crate::vm::method_lookup::WrittenReceiver;
use crate::vm::program::ArgumentList;
use crate::vm::statement::{PreparedTarget, ReceiverTarget};
use crate::vm::{ControlFlow, VirtualMachine};
use std::cell::RefCell;
use std::rc::Rc;

/// The operands an `Extend` supplies the high bits of.
#[derive(Default, Clone, Copy)]
struct Extension {
    high: usize,
    wide: usize,
}

impl Extension {
    fn register(self, operand: u8) -> usize {
        self.high * 256 + usize::from(operand)
    }

    fn index(self, operand: u16) -> usize {
        (self.wide << 16) | usize::from(operand)
    }
}

/// Where the loop goes after an instruction.
pub(crate) enum Next {
    Step,
    JumpTo(usize),
    Finish(ControlFlow),
}

/// One run of a unit: what it runs, the registers and state it runs over,
/// and the high operand bits an `Extend` left for the next instruction.
pub(crate) struct UnitRun<'a> {
    unit: &'a Unit,
    state: &'a mut RunState,
    registers: &'a mut [Object],
    base: OpenAtHandler,
    extension: Extension,
}

impl UnitRun<'_> {
    /// Where the registers start, which machine code reads and writes
    /// through.
    pub(crate) fn registers_pointer(&mut self) -> *mut Object {
        self.registers.as_mut_ptr()
    }

    pub(crate) fn unit(&self) -> &Unit {
        self.unit
    }

    pub(crate) fn register(&self, register: usize) -> Object {
        self.registers[register].clone()
    }

    pub(crate) fn set_register(&mut self, register: usize, value: Object) {
        self.registers[register] = value;
    }
}

/// How many emptied register lists the machine keeps for the next units it
/// runs.
const SPARE_REGISTER_LISTS: usize = 64;

/// What one run of a unit keeps besides its registers.
struct RunState {
    /// The cells the unit's locals are held in, shared with the scope the
    /// unit runs in, so `binding`, `local_variables`, and `eval` read and
    /// write them too. A slot holds its cell from the unit's first write of
    /// the local on.
    cells: Vec<Option<Rc<RefCell<Object>>>>,
    /// The argument lists being gathered, the innermost call's last.
    arguments: Vec<ArgumentList>,
    /// The errors handlers keep while a `rescue` or an `ensure` deals with
    /// them, by slot.
    errors: Vec<Option<MetorexError>>,
    /// The routes decided for calls whose arguments are being evaluated,
    /// the innermost call's last, each with whether the call was written
    /// with no receiver.
    routes: Vec<(CallRoute, bool)>,
}

impl VirtualMachine {
    /// Run `unit`, answering what its `Return` hands back. A `break`,
    /// `next`, or `redo` that ends the unit is refused, as at the top level.
    pub fn run_unit(&mut self, unit: &Unit) -> Result<Object, MetorexError> {
        match self.run_unit_flow(unit)? {
            ControlFlow::Value(value) => Ok(value),
            ControlFlow::Break { position, .. } => Err(loop_control_error("break", position)),
            ControlFlow::Continue { position, .. } => Err(loop_control_error("next", position)),
            ControlFlow::Redo { position } => Err(loop_control_error("redo", position)),
            _ => Ok(Object::Nil),
        }
    }

    /// Run `unit`, answering how it ended: with the value its `Return`
    /// hands back, or with a `break`, `next`, or `redo` written outside any
    /// loop of the unit, which the body the unit is decides what to do
    /// with.
    pub(crate) fn run_unit_flow(&mut self, unit: &Unit) -> Result<ControlFlow, MetorexError> {
        let mut state = RunState {
            cells: vec![None; unit.locals.len()],
            arguments: Vec::new(),
            routes: Vec::new(),
            errors: (0..unit.error_slots).map(|_| None).collect(),
        };
        // A continuation taken in the unit runs it again from the statement
        // its `callcc` was written in.
        self.run_restartable(|vm, statement| {
            // The first run starts ahead of the first statement, where the
            // unit's `BEGIN` bodies are.
            let at = match statement {
                0 => 0,
                _ => unit.statement_at(statement).unwrap_or(0),
            };
            // A loop or a `case` arm the run leaves early, by an error or a
            // jump, leaves what it opened for the run to close.
            let scopes = vm.environment().current_depth();
            let loops = vm.running_block_breaks.len();
            let ran = vm.run_unit_from(unit, &mut state, at);
            while vm.environment().current_depth() > scopes {
                vm.environment_mut().pop_scope();
            }
            vm.running_block_breaks.truncate(loops);
            ran
        })
    }

    fn run_unit_from(
        &mut self,
        unit: &Unit,
        state: &mut RunState,
        start: usize,
    ) -> Result<ControlFlow, MetorexError> {
        let mut registers = self.spare_registers.pop().unwrap_or_default();
        registers.resize(unit.register_count, Object::Nil);
        let ran = self.run_unit_in(unit, state, start, &mut registers);
        registers.clear();
        if self.spare_registers.len() < SPARE_REGISTER_LISTS {
            self.spare_registers.push(registers);
        }
        ran
    }

    fn run_unit_in(
        &mut self,
        unit: &Unit,
        state: &mut RunState,
        start: usize,
        registers: &mut [Object],
    ) -> Result<ControlFlow, MetorexError> {
        // What the run had open as it began, which a handler goes back to
        // with what was open where its instructions begin added.
        let base = OpenAtHandler {
            lists: state.arguments.len(),
            routes: state.routes.len(),
            scopes: self.environment().current_depth(),
            loops: self.running_block_breaks.len(),
        };
        let mut run = UnitRun {
            unit,
            state,
            registers,
            base,
            extension: Extension::default(),
        };
        let observing = crate::vm::jit::enabled();
        let mut at = start;
        let mut code = crate::vm::jit::code_for(self, unit);
        loop {
            if let Some(compiled) = code.take() {
                match crate::vm::jit::run(self, &compiled, &mut run, at) {
                    crate::vm::jit::Ran::Finished(ended) => return ended,
                    crate::vm::jit::Ran::Deoptimized(resume) => at = resume,
                }
            }
            match self.step(&mut run, at, observing)? {
                Next::Step => at += 1,
                Next::JumpTo(target) => {
                    // A loop going back counts as a run of its unit, so a
                    // loop in a unit run once is compiled too, and goes on
                    // in machine code from where it goes back to.
                    if observing && target <= at {
                        code = crate::vm::jit::code_for(self, unit);
                    }
                    at = target;
                }
                Next::Finish(flow) => return Ok(flow),
            }
        }
    }

    /// Run the instruction at `at`, with the statement it starts, if any,
    /// and the handler that takes an error it raises. When `observing`, an
    /// arithmetic instruction's operands are recorded for the JIT.
    #[inline(always)]
    pub(crate) fn step(
        &mut self,
        run: &mut UnitRun,
        at: usize,
        observing: bool,
    ) -> Result<Next, MetorexError> {
        let unit = run.unit;
        let instruction = unit.instructions[at];
        let position = unit.positions[at];
        if crate::vm::counters::enabled() {
            crate::vm::counters::count_instruction(instruction.name());
        }
        if let Instruction::Extend { high, wide } = instruction {
            run.extension = Extension {
                high: usize::from(high),
                wide: usize::from(wide),
            };
            return Ok(Next::Step);
        }
        let extension = std::mem::take(&mut run.extension);
        if observing {
            crate::vm::jit::observe(unit, at, instruction, run.registers, extension.high);
        }
        let ran = match unit.statement_starts[at] {
            Some(statement) => self
                .begin_unit_statement(run.registers, statement)
                .and_then(|()| {
                    self.execute_instruction(
                        unit,
                        run.registers,
                        run.state,
                        instruction,
                        extension,
                        position,
                    )
                }),
            None => self.execute_instruction(
                unit,
                run.registers,
                run.state,
                instruction,
                extension,
                position,
            ),
        };
        self.settle_instruction(run, at, ran)
    }

    /// What starting the statement whose first instruction is about to run
    /// does before that instruction.
    #[inline(always)]
    fn begin_unit_statement(
        &mut self,
        registers: &mut [Object],
        statement: StatementStart,
    ) -> Result<(), MetorexError> {
        // No temporary lives from one statement to the next, and one left
        // holding a value would keep it from being freed.
        for register in registers.iter_mut().skip(statement.live) {
            *register = Object::Nil;
        }
        if let Some(index) = statement.index {
            self.mark_statement(index);
        }
        self.start_statement(statement)
    }

    /// Where the run goes after the instruction at `at` ran, handing an
    /// error it raised to the handler that takes it.
    #[inline(always)]
    fn settle_instruction(
        &mut self,
        run: &mut UnitRun,
        at: usize,
        ran: Result<Next, MetorexError>,
    ) -> Result<Next, MetorexError> {
        match ran {
            Err(error) => {
                let unit = run.unit;
                if let Some(spot) = &unit.error_spots[at] {
                    self.note_compiled_error_spot(&error, spot);
                }
                match self.handle_error(unit, at, error, run.registers, run.state, run.base)? {
                    Some(target) => Ok(Next::JumpTo(target)),
                    None => unreachable!("an error no handler took is answered above"),
                }
            }
            other => other,
        }
    }

    /// Start the statement that the instruction at `at` starts, for
    /// machine code that runs the instruction itself afterwards.
    pub(crate) fn step_statement(
        &mut self,
        run: &mut UnitRun,
        at: usize,
    ) -> Result<Next, MetorexError> {
        let statement = run.unit.statement_starts[at]
            .expect("machine code starts a statement only where one starts");
        let ran = self
            .begin_unit_statement(run.registers, statement)
            .map(|()| Next::Step);
        self.settle_instruction(run, at, ran)
    }

    /// Run the instruction at `at` without the statement it starts, which
    /// machine code has started already. It never follows an `Extend`.
    pub(crate) fn step_instruction(
        &mut self,
        run: &mut UnitRun,
        at: usize,
    ) -> Result<Next, MetorexError> {
        let unit = run.unit;
        let ran = self.execute_instruction(
            unit,
            run.registers,
            run.state,
            unit.instructions[at],
            Extension::default(),
            unit.positions[at],
        );
        self.settle_instruction(run, at, ran)
    }

    /// Run the `GetLocal` at `at`, reading its cell when the run holds one.
    pub(crate) fn step_get_local(
        &mut self,
        run: &mut UnitRun,
        at: usize,
    ) -> Result<Next, MetorexError> {
        if let Instruction::GetLocal { to, slot } = run.unit.instructions[at]
            && let Some(cell) = &run.state.cells[usize::from(slot)]
        {
            run.registers[usize::from(to)] = cell.borrow().clone();
            return Ok(Next::Step);
        }
        self.step_instruction(run, at)
    }

    /// Run the `SetLocal` at `at`, writing its cell when the run holds one.
    pub(crate) fn step_set_local(
        &mut self,
        run: &mut UnitRun,
        at: usize,
    ) -> Result<Next, MetorexError> {
        if let Instruction::SetLocal { from, slot } = run.unit.instructions[at]
            && let Some(cell) = &run.state.cells[usize::from(slot)]
        {
            let value = run.registers[usize::from(from)].clone();
            *cell.borrow_mut() = value;
            return Ok(Next::Step);
        }
        self.step_instruction(run, at)
    }

    /// Run the `Unhoist` at `at`.
    pub(crate) fn step_unhoist(
        &mut self,
        run: &mut UnitRun,
        at: usize,
    ) -> Result<Next, MetorexError> {
        if let Instruction::Unhoist { slot } = run.unit.instructions[at] {
            let name = &run.unit.locals[usize::from(slot)].text;
            self.environment_mut().unhoist(name);
            return Ok(Next::Step);
        }
        self.step_instruction(run, at)
    }

    /// Run the `GetIdentifier` at `at`.
    pub(crate) fn step_get_identifier(
        &mut self,
        run: &mut UnitRun,
        at: usize,
    ) -> Result<Next, MetorexError> {
        if let Instruction::GetIdentifier { to, name } = run.unit.instructions[at] {
            let name = &run.unit.names[usize::from(name)].text;
            let ran = self
                .read_identifier(name, run.unit.positions[at])
                .map(|value| {
                    run.registers[usize::from(to)] = value;
                    Next::Step
                });
            return self.settle_instruction(run, at, ran);
        }
        self.step_instruction(run, at)
    }

    /// Run the `Checkpoint` at `at`.
    pub(crate) fn step_checkpoint(
        &mut self,
        run: &mut UnitRun,
        at: usize,
    ) -> Result<Next, MetorexError> {
        let ran = self
            .pass_checkpoint(run.unit.positions[at])
            .map(|()| Next::Step);
        self.settle_instruction(run, at, ran)
    }

    fn execute_instruction(
        &mut self,
        unit: &Unit,
        registers: &mut [Object],
        state: &mut RunState,
        instruction: Instruction,
        extension: Extension,
        position: Position,
    ) -> Result<Next, MetorexError> {
        let register = |operand: u8| extension.register(operand);
        let index = |operand: u16| extension.index(operand);
        let read = |registers: &[Object], operand: u8| match Operand::read(operand, extension.high)
        {
            Operand::Register(at) => registers[at].clone(),
            Operand::Constant(at) => unit.constants[at].clone(),
        };
        match instruction {
            Instruction::Extend { .. } => {}
            Instruction::Move { to, from } => {
                registers[register(to)] = registers[register(from)].clone();
            }
            Instruction::LoadConstant { to, constant } => {
                registers[register(to)] = unit.constants[index(constant)].clone();
            }
            Instruction::LoadInteger { to, value } => {
                registers[register(to)] = Object::Int(i64::from(value));
            }
            Instruction::LoadNil { to } => registers[register(to)] = Object::Nil,
            Instruction::LoadTrue { to } => registers[register(to)] = Object::Bool(true),
            Instruction::LoadFalse { to } => registers[register(to)] = Object::Bool(false),
            Instruction::LoadSelf { to } => registers[register(to)] = self.eval_self(position)?,
            Instruction::LoadSymbol { to, name } => {
                let name = &unit.names[index(name)].text;
                registers[register(to)] = self.evaluate_symbol_literal(name);
            }
            Instruction::NewString { to, text } => {
                let text = &unit.names[index(text)].text;
                registers[register(to)] = self.evaluate_string_literal(text, position);
            }
            Instruction::LoadRegex { to, pattern } => {
                let regex = &unit.regexes[index(pattern)];
                registers[register(to)] =
                    self.evaluate_regex_literal(&regex.pattern, &regex.flags, position);
            }
            Instruction::LoadFile { to } => registers[register(to)] = self.magic_file(),
            Instruction::LoadDirectory { to } => registers[register(to)] = self.magic_dir(),
            Instruction::Add { to, left, right }
            | Instruction::Subtract { to, left, right }
            | Instruction::Multiply { to, left, right }
            | Instruction::Divide { to, left, right }
            | Instruction::Modulo { to, left, right }
            | Instruction::Equal { to, left, right }
            | Instruction::NotEqual { to, left, right }
            | Instruction::Less { to, left, right }
            | Instruction::LessEqual { to, left, right }
            | Instruction::Greater { to, left, right }
            | Instruction::GreaterEqual { to, left, right } => {
                let left = read(registers, left);
                let right = read(registers, right);
                registers[register(to)] =
                    match self.specialized_operation(&instruction, &left, &right) {
                        Some(value) => value,
                        None => {
                            let operator = binary_operator(instruction);
                            self.operate_on_values(&operator, left, right, &position)?
                        }
                    };
            }
            Instruction::BinaryOperator { to, first, op } => {
                let first = register(first);
                let operator = &BINARY_OPERATORS[usize::from(op)];
                let (left, right) = (registers[first].clone(), registers[first + 1].clone());
                registers[register(to)] =
                    self.operate_on_values(operator, left, right, &position)?;
            }
            Instruction::Negate { to, from } => {
                let value = registers[register(from)].clone();
                registers[register(to)] =
                    self.apply_unary_operator(&UnaryOp::Minus, value, position)?;
            }
            Instruction::Positive { to, from } => {
                let value = registers[register(from)].clone();
                registers[register(to)] =
                    self.apply_unary_operator(&UnaryOp::Plus, value, position)?;
            }
            Instruction::Not { to, from } => {
                let value = registers[register(from)].clone();
                registers[register(to)] =
                    self.apply_unary_operator(&UnaryOp::Not, value, position)?;
            }
            Instruction::Jump { target } => return Ok(Next::JumpTo(index(target))),
            Instruction::JumpIfTrue { test, target } => {
                if registers[register(test)].view().is_truthy() {
                    return Ok(Next::JumpTo(index(target)));
                }
            }
            Instruction::JumpIfFalse { test, target } => {
                if !registers[register(test)].view().is_truthy() {
                    return Ok(Next::JumpTo(index(target)));
                }
            }
            Instruction::JumpIfNil { test, target } => {
                if registers[register(test)].is_nil() {
                    return Ok(Next::JumpTo(index(target)));
                }
            }
            Instruction::Call { base, site } => {
                let site = &unit.call_sites[index(site)];
                let base = register(base);
                let receiver = registers[base].clone();
                let (arguments, block) = call_inputs(site, base, registers, state);
                registers[base] = match site.receiver {
                    Receiver::Sent => {
                        self.send_to_object(receiver, &site.name, arguments, position)?
                    }
                    _ => self.call_from_unit(receiver, site, arguments, block, position)?,
                };
            }
            Instruction::StartArguments => state.arguments.push(ArgumentList::default()),
            Instruction::PushArgument { from } => {
                let value = registers[register(from)].clone();
                open_list(state).push(value);
            }
            Instruction::SplatArgument { from } => {
                let value = registers[register(from)].clone();
                open_list(state).splat(self, value, position)?;
            }
            Instruction::KeywordSplatArgument { from } => {
                let value = registers[register(from)].clone();
                open_list(state).keyword_splat(self, value, position)?;
            }
            Instruction::BlockArgument { from } => {
                let value = registers[register(from)].clone();
                open_list(state).block_argument(self, value, position)?;
            }
            Instruction::ArgumentsToArray { to } => {
                let list = state.arguments.pop().expect("an argument list was begun");
                registers[register(to)] = Object::array(list.finish());
            }
            Instruction::ResolveCall { site } => {
                let site = &unit.call_sites[index(site)];
                let route = self.resolve_bare_call(
                    &site.name,
                    site.written_with_arguments,
                    site.has_block,
                    site.keyword_splat,
                    position,
                )?;
                state.routes.push((route, true));
            }
            Instruction::ResolveScopedCall { namespace, site } => {
                let site = &unit.call_sites[index(site)];
                let owner = registers[register(namespace)].clone();
                let route = if matches!(
                    self.lookup_method(&owner, &site.name),
                    Some((_, found)) if !found.is_undefined
                ) {
                    CallRoute::OnReceiver {
                        receiver: owner,
                        written: written_receiver(site.receiver),
                    }
                } else {
                    match self.read_scope_resolution(owner, &site.name, position) {
                        Ok(value) => CallRoute::Callable(value),
                        Err(error) => CallRoute::Failed(error),
                    }
                };
                state.routes.push((route, false));
            }
            Instruction::CallResolved { base, site } => {
                let site = &unit.call_sites[index(site)];
                let base = register(base);
                let (route, bare) = state.routes.pop().expect("a call route was decided");
                let (arguments, block) = call_inputs(site, base, registers, state);
                crate::vm::program::enter_nesting(position)?;
                let attached = self.attached_block_flags.len();
                let result = self.run_call_route(
                    route,
                    &site.name,
                    arguments,
                    block,
                    position,
                    Some(&site.cache),
                );
                self.release_attached_blocks(attached);
                crate::vm::program::leave_nesting();
                if bare {
                    self.note_failed_call(&result, &site.name, true, position);
                }
                registers[base] = result?;
            }
            Instruction::IsNativeName { to, name } => {
                let name = &unit.names[index(name)].text;
                let native = self
                    .environment()
                    .get(name)
                    .is_some_and(|held| matches!(held.view(), View::NativeFunction(_)));
                registers[register(to)] = Object::Bool(native);
            }
            Instruction::CallNativeName { base, site } => {
                let site = &unit.call_sites[index(site)];
                let base = register(base);
                let (arguments, _) = call_inputs(site, base, registers, state);
                let native =
                    self.environment()
                        .get(&site.name)
                        .and_then(|held| match held.view() {
                            View::NativeFunction(native) => Some(native.to_string()),
                            _ => None,
                        });
                let Some(native) = native else {
                    return Err(MetorexError::runtime_error(
                        format!("{} is no longer a Kernel function", site.name),
                        crate::vm::utils::position_to_location(position),
                    ));
                };
                registers[base] =
                    self.call_native_function(&native, vec![Object::array(arguments)], position)?;
            }
            Instruction::SafeNavigation { test, target } => {
                let missing = registers[register(test)].is_nil();
                self.coverage_branch(position, usize::from(missing));
                if missing {
                    return Ok(Next::JumpTo(index(target)));
                }
            }
            Instruction::MakeBlock { to, literal } => {
                registers[register(to)] = self.evaluate_block_literal(
                    &unit.blocks[index(literal)],
                    &unit.block_bodies[index(literal)],
                )?;
            }
            Instruction::SplatInto { array, from } => {
                let spread = self.splat_elements(registers[register(from)].clone(), position)?;
                if let View::Array(elements) = registers[register(array)].view() {
                    elements.borrow_mut().extend(spread);
                }
            }
            Instruction::IndexSpread {
                to,
                collection,
                subscripts,
            } => {
                let collection = registers[register(collection)].clone();
                let subscripts = match registers[register(subscripts)].view() {
                    View::Array(elements) => elements.borrow().clone(),
                    _ => Vec::new(),
                };
                registers[register(to)] =
                    self.send_to_object(collection, "[]", subscripts, position)?;
            }
            Instruction::MatchPattern {
                register: held,
                pattern,
            } => {
                let (pattern, refuses) = &unit.patterns[index(pattern)];
                let value = registers[register(held)].clone();
                registers[register(held)] =
                    self.test_pattern(value, pattern, *refuses, position)?;
            }
            Instruction::DeclareLocal { slot } => {
                let name = &unit.locals[index(slot)].text;
                if self.environment().get(name).is_none() {
                    self.environment_mut().define(name.clone(), Object::Nil);
                } else {
                    // A `for` loop names what its body binds so the scope
                    // holding the loop still reads it afterwards, which
                    // means the block the loop runs closes over the name.
                    self.environment_mut().unhoist(name);
                }
            }
            Instruction::Unhoist { slot } => {
                let name = &unit.locals[index(slot)].text;
                self.environment_mut().unhoist(name);
            }
            Instruction::GetLocal { to, slot } => {
                let slot = index(slot);
                registers[register(to)] = match &state.cells[slot] {
                    Some(cell) => cell.borrow().clone(),
                    None => self.read_identifier(&unit.locals[slot].text, position)?,
                };
            }
            Instruction::SetLocal { from, slot } => {
                let slot = index(slot);
                let value = registers[register(from)].clone();
                match &state.cells[slot] {
                    Some(cell) => *cell.borrow_mut() = value,
                    None => {
                        let name = unit.locals[slot].text.clone();
                        let target = Expression::Identifier {
                            name: name.clone(),
                            position,
                        };
                        self.assign_value(&target, value)?;
                        state.cells[slot] = self.environment().get_ref(&name);
                    }
                }
            }
            Instruction::GetIdentifier { to, name } => {
                let name = &unit.names[index(name)].text;
                registers[register(to)] = self.read_identifier(name, position)?;
            }
            Instruction::GetInstanceVariable { to, name } => {
                let at = index(name);
                let name = &unit.names[at].text;
                registers[register(to)] = match self.read_slot(name, &unit.slot_caches[at]) {
                    Some(value) => value,
                    None => self.eval_instance_var_read(name, position)?,
                };
            }
            Instruction::SetInstanceVariable { from, name } => {
                let at = index(name);
                let value = registers[register(from)].clone();
                if let Some(value) =
                    self.write_slot(&unit.names[at].text, &unit.slot_caches[at], value)
                {
                    let name = unit.names[at].text.clone();
                    let target = Expression::InstanceVariable { name, position };
                    self.assign_value(&target, value)?;
                }
            }
            Instruction::GetConstant { to, name } => {
                let at = index(name);
                registers[register(to)] = self.read_constant_through(
                    &unit.names[at].text,
                    &unit.constant_caches[at],
                    position,
                )?;
            }
            Instruction::SetConstant { from, name } => {
                let name = unit.names[index(name)].text.clone();
                let target = Expression::Identifier { name, position };
                self.assign_value(&target, registers[register(from)].clone())?;
            }
            Instruction::GetTopLevelConstant { to, name } => {
                let name = &unit.names[index(name)].text;
                registers[register(to)] = self.read_top_level_constant(name, position)?;
            }
            Instruction::GetScopedConstant { namespace, name } => {
                let name = &unit.names[index(name)].text;
                let held = registers[register(namespace)].clone();
                registers[register(namespace)] =
                    self.read_scope_resolution(held, name, position)?;
            }
            Instruction::GetClassVariable { to, name } => {
                let name = &unit.names[index(name)].text;
                registers[register(to)] = self.eval_class_var_read(name, position)?;
            }
            Instruction::SetClassVariable { from, name } => {
                let name = unit.names[index(name)].text.clone();
                let target = Expression::ClassVariable { name, position };
                self.assign_value(&target, registers[register(from)].clone())?;
            }
            Instruction::GetGlobal { to, name } => {
                let name = &unit.names[index(name)].text;
                registers[register(to)] = self.read_global_variable(name, position)?;
            }
            Instruction::SetGlobal { from, name } => {
                let name = &unit.names[index(name)].text;
                self.write_global_variable(name, registers[register(from)].clone(), position)?;
            }
            Instruction::NewArray { to, first, count } => {
                let first = register(first);
                let elements = registers[first..first + usize::from(count)].to_vec();
                let made = Object::array(elements);
                self.record_allocation(&made, position);
                registers[register(to)] = made;
            }
            Instruction::AppendArray {
                array,
                first,
                count,
            } => {
                let first = register(first);
                if let View::Array(elements) = registers[register(array)].view() {
                    elements
                        .borrow_mut()
                        .extend_from_slice(&registers[first..first + usize::from(count)]);
                }
            }
            Instruction::AddToHash { hash, shape } => {
                let literal = HashLiteral::resume(&registers[register(hash)]);
                registers[register(hash)] =
                    self.build_hash(literal, &unit.hashes[index(shape)], registers)?;
            }
            Instruction::NewHash { to, shape } => {
                let made = self.build_hash(
                    HashLiteral::default(),
                    &unit.hashes[index(shape)],
                    registers,
                )?;
                self.record_allocation(&made, position);
                registers[register(to)] = made;
            }
            Instruction::NewRange { to, first, flags } => {
                let first = register(first);
                registers[register(to)] = self.build_range(
                    registers[first].clone(),
                    registers[first + 1].clone(),
                    flags & RANGE_EXCLUSIVE != 0,
                    flags & RANGE_LITERAL_ENDS != 0,
                    position,
                )?;
            }
            Instruction::ToText { to, from } => {
                let text = self.interpolated_text(&registers[register(from)], position)?;
                registers[register(to)] = Object::string(text);
            }
            Instruction::Interpolate { to, shape } => {
                let mut interpolation = Interpolation::default();
                for piece in &unit.interpolations[index(shape)] {
                    match piece {
                        Piece::Text(text) => interpolation.push_text(text),
                        Piece::Value {
                            register: held,
                            position: at,
                        } => {
                            let written = match registers[held + 1].view() {
                                View::String(text) => text.as_str().to_string(),
                                other => other.to_string(),
                            };
                            interpolation.push_value(&registers[*held], &written, *at)?;
                        }
                    }
                }
                registers[register(to)] = interpolation.finish(self);
            }
            Instruction::Index {
                to,
                collection,
                key,
            } => {
                let collection = registers[register(collection)].clone();
                let key = registers[register(key)].clone();
                registers[register(to)] = self.index_value(collection, key, position)?;
            }
            Instruction::StoreTarget { from, target } => {
                let target = prepared(&unit.targets[index(target)], registers);
                self.store_prepared(&target, registers[register(from)].clone())?;
            }
            Instruction::ReadTarget { to, target } => {
                let shape = &unit.targets[index(target)];
                let or_assigning = matches!(
                    shape,
                    Target::ScopedConstant {
                        or_assigning: true,
                        ..
                    }
                );
                let PreparedTarget::Receiver(on_receiver) = prepared(shape, registers) else {
                    unreachable!("an operator assignment reads only a target on a receiver");
                };
                registers[register(to)] = self.read_on_receiver(&on_receiver, or_assigning)?;
            }
            Instruction::AssignMultiple { from, shape } => {
                let shape = &unit.multiple_assignments[index(shape)];
                let value = registers[register(from)].clone();
                let source = if shape.lone_value {
                    self.block_argument_spread(&value, shape.position)?
                        .unwrap_or_else(|| vec![value.clone()])
                } else {
                    match value.view() {
                        View::Array(elements) => elements.borrow().clone(),
                        _ => vec![value.clone()],
                    }
                };
                let targets: Vec<PreparedTarget> = shape
                    .targets
                    .iter()
                    .map(|target| prepared(target, registers))
                    .collect();
                self.spread_into_prepared(&targets, &source)?;
            }
            Instruction::IsDefined { to, expression } => {
                registers[register(to)] = self.eval_defined(&unit.defined[index(expression)])?;
            }
            Instruction::Raise { from } => {
                let raised = registers[register(from)].clone();
                self.raise_exception(vec![raised], position)?;
            }
            Instruction::Return { from } => {
                return Ok(Next::Finish(ControlFlow::Value(
                    registers[register(from)].clone(),
                )));
            }
            Instruction::ExitBreak { from } => {
                return Ok(Next::Finish(ControlFlow::Break {
                    value: registers[register(from)].clone(),
                    position,
                }));
            }
            Instruction::ExitNext { from } => {
                return Ok(Next::Finish(ControlFlow::Continue {
                    value: registers[register(from)].clone(),
                    position,
                }));
            }
            Instruction::ExitRedo => return Ok(Next::Finish(ControlFlow::Redo { position })),
            Instruction::ExitRetry => return Ok(Next::Finish(ControlFlow::Retry { position })),
            Instruction::BreakCheck { from } => {
                let value = registers[register(from)].clone();
                let location = crate::vm::utils::position_to_location(position);
                if let error @ MetorexError::UncaughtException { .. } =
                    self.break_signal(value, location)
                {
                    return Err(error);
                }
            }
            Instruction::SaveCurrentException { to } => {
                registers[register(to)] = self.globals().get("!").unwrap_or(Object::Nil);
            }
            Instruction::RestoreCurrentException { from } => {
                self.restore_current_exception(registers[register(from)].clone());
            }
            Instruction::RescueMatches {
                register: held,
                clause,
            } => {
                let held = register(held);
                let clause = &unit.rescue_clauses[index(clause)].clause;
                let count = clause.splatted_types.len();
                let spreads = registers[held + 2..held + 2 + count].to_vec();
                let exception = registers[held].clone();
                let takes =
                    self.rescue_clause_takes(clause, spreads, &exception, clause.position)?;
                registers[held + 1] = Object::Bool(takes);
            }
            Instruction::EnterRescue { from, clause } => {
                let shape = &unit.rescue_clauses[index(clause)];
                let exception = registers[register(from)].clone();
                self.enter_rescue(&shape.clause, exception, shape.expression_form)?;
            }
            Instruction::Reraise { slot } => {
                return Err(state.errors[index(slot)]
                    .take()
                    .expect("a handler kept the error"));
            }
            Instruction::RaiseStatement { from, written } => {
                let value = (written != 0).then(|| registers[register(from)].clone());
                let exception = self.statement_raise(value, position)?;
                return Err(MetorexError::UncaughtException {
                    message: crate::vm::utils::format_exception(&exception),
                    exception,
                    location: crate::vm::utils::position_to_location(position),
                });
            }
            Instruction::ExitReturn { from, written } => {
                if written != 0 {
                    self.warn_top_level_return_value(position);
                }
                return Ok(Next::Finish(ControlFlow::Return {
                    value: registers[register(from)].clone(),
                    position,
                }));
            }
            Instruction::FindBlock { to } => {
                registers[register(to)] = self.block_to_yield(position)?;
            }
            Instruction::CallBlock { base, site } => {
                let site = &unit.call_sites[index(site)];
                let base = register(base);
                let (arguments, _) = call_inputs(site, base, registers, state);
                let View::Block(block) = registers[base].view() else {
                    unreachable!("FindBlock puts a block in the register");
                };
                let block = Rc::clone(block);
                crate::vm::program::enter_nesting(position)?;
                let answer = block.call(self, arguments, position);
                crate::vm::program::leave_nesting();
                registers[base] = answer?;
            }
            Instruction::PushScope => self.environment_mut().push_scope(),
            Instruction::BeginBlockOnce { to } => {
                // A program read one line at a time runs the same unit once
                // per line, and the body belongs to the unit rather than to
                // each pass over it.
                let site = (
                    self.current_source_file.clone().unwrap_or_default(),
                    position.line,
                    position.column,
                );
                registers[register(to)] = Object::Bool(self.opened_blocks.insert(site));
            }
            Instruction::BodyStatement { to, statement } => {
                let body = self.running_body();
                let statement = &unit.body_statements[index(statement)];
                let mut last_value = registers[register(to)].clone();
                if body.module {
                    self.module_body_statement(&body.class, statement, &mut last_value)?;
                } else {
                    self.class_body_statement(
                        &body.class,
                        statement,
                        body.position,
                        &mut last_value,
                    )?;
                }
                registers[register(to)] = last_value;
            }
            Instruction::BodyConstant { from, name } => {
                let body = self.running_body();
                let name = &unit.names[index(name)].text;
                let value = registers[register(from)].clone();
                if body.module {
                    self.module_body_constant(&body.class, name, value, position)?;
                } else if self
                    .constant_home()
                    .is_some_and(|home| Rc::ptr_eq(&home, &body.class))
                {
                    self.class_body_constant(&body.class, name, value, position)?;
                } else {
                    // A block run as the body keeps its constants where the
                    // block was written, which the plain assignment handles.
                    let target = Expression::Identifier {
                        name: name.clone(),
                        position,
                    };
                    self.assign_value(&target, value)?;
                }
            }
            Instruction::BodyAttributes { to, names, kind } => {
                let body = self.running_body();
                let method = ["attr_reader", "attr_writer", "attr_accessor"][usize::from(kind)];
                let values = match registers[register(names)].view() {
                    View::Array(held) => held.borrow().clone(),
                    _ => Vec::new(),
                };
                if let Some(answer) =
                    self.class_body_attributes(&body.class, method, values, position)?
                {
                    registers[register(to)] = answer;
                }
            }
            Instruction::BodyInstanceVariable { from, name } => {
                let body = self.running_body();
                let name = &unit.names[index(name)].text;
                crate::vm::class_execution::class_body_instance_variable(
                    &body.class,
                    name,
                    registers[register(from)].clone(),
                );
            }
            Instruction::BodyClassVariable { from, name } => {
                let body = self.running_body();
                let name = &unit.names[index(name)].text;
                crate::vm::class_execution::class_body_class_variable(
                    &body.class,
                    name,
                    registers[register(from)].clone(),
                );
            }
            Instruction::BodyAliasMethod { first } => {
                let body = self.running_body();
                let first = register(first);
                let (new_value, old_value) =
                    (registers[first].clone(), registers[first + 1].clone());
                self.class_body_alias_method(&body.class, &new_value, &old_value, position)?;
            }
            Instruction::BodyDefineMethod { to, block, count } => {
                let body = self.running_body();
                let block = register(block);
                let arguments = registers[block + 1..block + 1 + usize::from(count)].to_vec();
                let made = registers[block].clone();
                if let View::Block(held) = made.view() {
                    held.attached_call_running.set(true);
                    self.attached_block_flags
                        .push(Rc::clone(&held.attached_call_running));
                }
                registers[register(to)] =
                    self.class_body_define_method(&body.class, &arguments, made, position)?;
            }
            Instruction::BodyBlockAccessors { names } => {
                let body = self.running_body();
                let values = match registers[register(names)].view() {
                    View::Array(held) => held.borrow().clone(),
                    _ => Vec::new(),
                };
                let mut names = Vec::with_capacity(values.len());
                for value in &values {
                    names.push(self.coerce_method_name(value, "attr_accessor", body.position)?);
                }
                crate::vm::class_execution::define_block_accessors(&body.class, &names);
            }
            Instruction::InvokeIfMethod { register: held } => {
                let value = registers[register(held)].clone();
                if matches!(value.view(), View::Method(_)) {
                    registers[register(held)] = self.invoke_callable(value, vec![], position)?;
                }
            }
            Instruction::ModuleBlockStatement { names, statement } => {
                let body = self.running_body();
                let statement = &unit.body_statements[index(statement)];
                let given = match statement {
                    Statement::AttrReader { .. }
                    | Statement::AttrWriter { .. }
                    | Statement::AttrAccessor { .. } => {
                        let values = match registers[register(names)].view() {
                            View::Array(held) => held.borrow().clone(),
                            _ => Vec::new(),
                        };
                        let mut names = Vec::with_capacity(values.len());
                        for value in &values {
                            names.push(self.coerce_method_name(
                                value,
                                "attr_accessor",
                                Position::default(),
                            )?);
                        }
                        Some(names)
                    }
                    _ => None,
                };
                self.module_block_statement(&body.class, statement, given)?;
            }
            Instruction::Define { to, definition } => {
                registers[register(to)] =
                    self.make_definition(&unit.definitions[index(definition)], registers)?;
            }
            Instruction::AttributeSplat { array, from } => {
                let spread = registers[register(from)].clone();
                let items = match self.splat_through_to_a(spread, position)? {
                    Ok(items) => items,
                    Err(held) => vec![held],
                };
                if let View::Array(names) = registers[register(array)].view() {
                    names.borrow_mut().extend(items);
                }
            }
            Instruction::AttachSuperBlock { from } => {
                if let View::Block(held) = registers[register(from)].view() {
                    held.attached_call_running.set(true);
                    self.attached_block_flags
                        .push(Rc::clone(&held.attached_call_running));
                }
            }
            Instruction::ClearSuperBlock { written } => {
                self.pending_block_from_ampersand = written != 0;
                self.pending_block = None;
            }
            Instruction::CallSuper { base, site } => {
                let site = &unit.call_sites[index(site)];
                let base = register(base);
                let (arguments, block) = call_inputs(site, base, registers, state);
                let attached_by_this_call = block
                    .as_ref()
                    .is_some_and(|held| matches!(held.view(), View::Block(_)));
                let attached = self.attached_block_flags.len() - usize::from(attached_by_this_call);
                let super_block = block.clone().or_else(|| self.block_super_passes());
                let arguments = if site.forwards_arguments {
                    SuperArguments::Forwarded
                } else {
                    SuperArguments::Written {
                        values: arguments,
                        block_argument: site.block_argument,
                    }
                };
                let calling_frame = self.current_method_frame;
                crate::vm::program::enter_nesting(position)?;
                let result = self.super_with(&arguments, super_block, position);
                self.release_attached_blocks(attached);
                crate::vm::program::leave_nesting();
                // `break` in the block written on the `super` returns from
                // the `super` call.
                registers[base] = match result {
                    Err(MetorexError::BlockBreak {
                        value, home_frame, ..
                    }) if block.is_some() && home_frame == calling_frame => value,
                    other => other?,
                };
            }
            Instruction::EnterLoop => self.running_block_breaks.push(None),
            Instruction::LeaveLoop => {
                self.running_block_breaks.pop();
            }
            Instruction::Checkpoint => self.pass_checkpoint(position)?,
            Instruction::CoverBranch { arm } => self.coverage_branch(position, usize::from(arm)),
            Instruction::CoverLine => {
                if self.coverage.is_some() {
                    self.coverage_count(position.line);
                }
            }
            Instruction::CaseArm { subject, arm } => {
                let subject = register(subject);
                let arm = &unit.case_arms[index(arm)];
                let matched = self.enter_case_arm(arm, &registers[subject])?;
                registers[subject + 1] = Object::Bool(matched);
            }
            Instruction::WarnDuplicatedWhen { list } => {
                let (cases, written_at) = &unit.when_lists[index(list)];
                self.warn_duplicated_when_clauses(cases, *written_at)?;
            }
            Instruction::WhenArm { subject, arm } => {
                let subject = register(subject);
                let arm = &unit.pattern_arms[index(arm)];
                let matched = self.enter_when_arm(arm, &registers[subject])?;
                registers[subject + 1] = Object::Bool(matched);
            }
            Instruction::InArm { subject, arm } => {
                let subject = register(subject);
                let arm = &unit.pattern_arms[index(arm)];
                let matched = self.enter_in_arm(arm, &registers[subject])?;
                registers[subject + 1] = Object::Bool(matched);
            }
            Instruction::GuardFailed => {
                self.note_pattern_failure(crate::vm::pattern_matching::PatternFailure::Detail(
                    "guard clause does not return true".to_string(),
                ));
            }
            Instruction::PushDeconstructed { from } => {
                self.deconstructed_values
                    .push((registers[register(from)].clone(), None));
            }
            Instruction::PopDeconstructed => {
                self.deconstructed_values.pop();
            }
            Instruction::NoMatchingPattern { subject, single } => {
                let value = registers[register(subject)].clone();
                return Err(if single != 0 {
                    self.detailed_no_matching_pattern(&value, position)
                } else {
                    self.no_matching_pattern(&value, position)
                });
            }
            Instruction::PopScope => {
                self.environment_mut().pop_scope();
            }
            Instruction::Unwind {
                lists,
                routes,
                scopes,
            } => {
                let lists = state.arguments.len().saturating_sub(usize::from(lists));
                state.arguments.truncate(lists);
                let routes = state.routes.len().saturating_sub(usize::from(routes));
                state.routes.truncate(routes);
                for _ in 0..scopes {
                    self.environment_mut().pop_scope();
                }
            }
            Instruction::FlipFlopState { to, flip } => {
                let (written_at, integer_sides) = unit.flip_flops[index(flip)];
                registers[register(to)] =
                    Object::Bool(self.flip_flop_running(written_at, integer_sides));
            }
            Instruction::FlipFlopSet { on, flip } => {
                self.set_flip_flop(unit.flip_flops[index(flip)].0, on != 0);
            }
            Instruction::LineIs { to, constant } => {
                let line = unit.constants[index(constant)].clone();
                let held = self.globals().get(".").unwrap_or(Object::Int(0));
                registers[register(to)] = Object::Bool(held == line);
            }
            Instruction::LastLine { to } => {
                registers[register(to)] = self.globals().get("_").unwrap_or(Object::Nil);
            }
        }
        Ok(Next::Step)
    }
}

impl VirtualMachine {
    /// The class or module body running as a compiled unit innermost.
    fn running_body(&self) -> crate::vm::units::BodyContext {
        self.body_contexts
            .last()
            .cloned()
            .expect("a class or module body is running")
    }

    /// Make a definition, with what it needed evaluated in the registers,
    /// answering what the statement answers.
    fn make_definition(
        &mut self,
        definition: &Definition,
        registers: &[Object],
    ) -> Result<Object, MetorexError> {
        let flow = match definition {
            Definition::Statement(statement) => self.run_definition(statement)?,
            Definition::Class {
                statement:
                    Statement::ClassDef {
                        name,
                        superclass,
                        body,
                        position,
                        ..
                    },
                namespace,
                superclass: superclass_value,
            } => self.define_class(
                name,
                namespace.map(|held| registers[held].clone()),
                superclass.as_deref(),
                superclass_value.map(|held| registers[held].clone()),
                body,
                *position,
            )?,
            Definition::Module {
                statement:
                    Statement::ModuleDef {
                        name,
                        body,
                        position,
                        ..
                    },
                namespace,
            } => self.define_module(
                name,
                namespace.map(|held| registers[held].clone()),
                body,
                *position,
            )?,
            Definition::SingletonClass {
                target,
                body,
                position,
            } => {
                return self.open_singleton_class(
                    registers[*target].clone(),
                    None,
                    body,
                    *position,
                );
            }
            Definition::Attributes {
                method,
                names,
                position,
            } => {
                let mut arguments = vec![Object::symbol(method.to_string())];
                if let View::Array(held) = registers[*names].view() {
                    arguments.extend(held.borrow().iter().cloned());
                }
                let receiver = self.environment().get("self").unwrap_or(Object::Nil);
                return self.send_to_object(receiver, "__send__", arguments, *position);
            }
            Definition::Class { .. } | Definition::Module { .. } => {
                unreachable!("a class or module definition holds its statement")
            }
        };
        Ok(match flow {
            ControlFlow::Value(value) => value,
            _ => Object::Nil,
        })
    }

    /// Take an error raised at `at` to the innermost handler covering it
    /// that takes it, answering where the run goes on, or answer the error
    /// when none does.
    fn handle_error(
        &mut self,
        unit: &Unit,
        at: usize,
        error: MetorexError,
        registers: &mut [Object],
        state: &mut RunState,
        base: OpenAtHandler,
    ) -> Result<Option<usize>, MetorexError> {
        for handler in unit.handlers_for(at) {
            match handler.kind {
                HandlerKind::Rescue { slot } => {
                    let Some((exception, raised_at)) =
                        crate::vm::exceptions::rescuable_exception(&error)
                    else {
                        continue;
                    };
                    self.close_back_to(handler.open, base, state);
                    self.note_exception_location(
                        &exception,
                        &crate::vm::utils::position_to_location(raised_at),
                    );
                    self.set_current_exception(exception.clone());
                    registers[handler.register] = exception;
                    if let Some(kept) = state.errors.get_mut(slot) {
                        *kept = Some(error);
                    }
                    return Ok(Some(handler.target));
                }
                HandlerKind::Ensure { slot } => {
                    self.close_back_to(handler.open, base, state);
                    // An exception on its way out is what `$!` names while
                    // the `ensure` clause runs, and what an exception raised
                    // there takes as its cause.
                    if let Some((exception, raised_at)) =
                        crate::vm::exceptions::rescuable_exception(&error)
                    {
                        self.note_exception_location(
                            &exception,
                            &crate::vm::utils::position_to_location(raised_at),
                        );
                        self.set_current_exception(exception);
                    }
                    state.errors[slot] = Some(error);
                    return Ok(Some(handler.target));
                }
                HandlerKind::Cleanup { slot } => {
                    self.close_back_to(handler.open, base, state);
                    state.errors[slot] = Some(error);
                    return Ok(Some(handler.target));
                }
                HandlerKind::RescueBody {
                    handled,
                    standing,
                    expression_form,
                } => {
                    // An exception raised by the rescue body takes the one
                    // being handled as its cause.
                    if let MetorexError::UncaughtException {
                        exception: raised, ..
                    } = &error
                    {
                        Self::record_cause(raised, &registers[handled]);
                        if expression_form {
                            // A clause that raised leaves `$!` naming what
                            // it raised.
                            self.set_current_exception(raised.clone());
                            continue;
                        }
                    }
                    self.restore_current_exception(registers[standing].clone());
                }
            }
        }
        Err(error)
    }

    /// Close what the run opened past what `open` names, counted from what
    /// it had open as it began.
    fn close_back_to(&mut self, open: OpenAtHandler, base: OpenAtHandler, state: &mut RunState) {
        state.arguments.truncate(base.lists + open.lists);
        state.routes.truncate(base.routes + open.routes);
        while self.environment().current_depth() > base.scopes + open.scopes {
            self.environment_mut().pop_scope();
        }
        self.running_block_breaks.truncate(base.loops + open.loops);
    }

    /// What happens as a statement starts.
    fn start_statement(&mut self, statement: StatementStart) -> Result<(), MetorexError> {
        let line = statement.position.line;
        match statement.kind {
            StartKind::Statement { line_event } => {
                self.begin_statement(statement.position, line_event)
            }
            StartKind::Terminal { line_event } => {
                if line_event && !self.tracepoints.is_empty() {
                    self.fire_line_event(statement.position)?;
                }
                if self.coverage.is_some() {
                    self.coverage_count(line);
                }
                Ok(())
            }
            StartKind::BodyStatement { runs } => {
                if self.coverage.is_some() {
                    self.coverage_count(line);
                    self.coverage_skip_line = Some(line);
                }
                if runs {
                    self.begin_statement(statement.position, true)?;
                }
                Ok(())
            }
            StartKind::InBranch { enclosing_line } => {
                if self.coverage.is_some() && line != enclosing_line {
                    self.coverage_count(line);
                }
                Ok(())
            }
        }
    }

    /// The Hash `literal` makes once the entries of the registers `shape`
    /// lays out are added.
    fn build_hash(
        &mut self,
        mut literal: HashLiteral,
        shape: &HashShape,
        registers: &[Object],
    ) -> Result<Object, MetorexError> {
        let mut at = shape.first;
        for entry in &shape.entries {
            match entry {
                HashEntry::Spread { position } => {
                    literal.spread(self, registers[at].clone(), *position)?;
                }
                HashEntry::Pair {
                    key_position,
                    value_line,
                    literal_key,
                } => {
                    let key = literal.key(self, registers[at].clone(), *key_position)?;
                    let written = WrittenPair {
                        key_position: *key_position,
                        value_line: *value_line,
                        literal_key: *literal_key,
                    };
                    literal.store(self, key, registers[at + 1].clone(), &written)?;
                }
            }
            at += entry.register_count();
        }
        Ok(literal.finish())
    }

    /// Call the method a call site names, with what is done around any
    /// method call.
    /// The instance variable `name` of `self` when `self` is an instance,
    /// read from its slot.
    /// The value in `slot` of `self`'s instance variables, when `self` is an
    /// instance whose variables have `shape`.
    pub(crate) fn read_slot_of_shape(
        &self,
        shape: *const crate::object::shape::Shape,
        slot: usize,
    ) -> Option<Object> {
        let held = self.environment().get("self")?;
        let View::Instance(instance) = held.view() else {
            return None;
        };
        let instance = instance.borrow();
        let variables = &instance.instance_vars;
        std::ptr::eq(Rc::as_ptr(variables.shape()), shape).then(|| variables.slot(slot).clone())
    }

    fn read_slot(&self, name: &str, cache: &SlotCache) -> Option<Object> {
        let held = self.environment().get("self")?;
        let View::Instance(instance) = held.view() else {
            return None;
        };
        let instance = instance.borrow();
        let variables = &instance.instance_vars;
        Some(match cache.slot_in(variables, name) {
            Some(slot) => variables.slot(slot).clone(),
            None => Object::Nil,
        })
    }

    /// Write `value` into the slot of the instance variable `name` of
    /// `self`, when `self` is an instance that holds it already and may be
    /// changed. Hands the value back otherwise.
    fn write_slot(&self, name: &str, cache: &SlotCache, value: Object) -> Option<Object> {
        let Some(held) = self.environment().get("self") else {
            return Some(value);
        };
        let View::Instance(instance) = held.view() else {
            return Some(value);
        };
        let mut instance = instance.borrow_mut();
        // A program's own variable named like the interpreter's slots is
        // recorded as the program's when it is written.
        if instance.frozen || name.starts_with("__") {
            return Some(value);
        }
        match cache.slot_in(&instance.instance_vars, name) {
            Some(slot) => {
                instance.instance_vars.set_slot(slot, value);
                None
            }
            None => Some(value),
        }
    }

    fn call_from_unit(
        &mut self,
        receiver: Object,
        site: &CallSite,
        arguments: Vec<Object>,
        block: Option<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        crate::vm::program::enter_nesting(position)?;
        let attached = self.attached_block_flags.len();
        let written = written_receiver(site.receiver);
        let result = if site.sends_literal_name && self.send_in_force(&receiver, &site.name) {
            self.send_literal_name(receiver, &site.name, arguments, block, position)
        } else {
            self.call_with_block_through(
                receiver,
                &site.name,
                arguments,
                block,
                written,
                position,
                Some(&site.cache),
            )
        };
        self.release_attached_blocks(attached);
        crate::vm::program::leave_nesting();
        self.note_failed_call(&result, &site.name, written.names_self(), position);
        result
    }
}

/// How method lookup names the way a call's receiver was written.
fn written_receiver(receiver: Receiver) -> WrittenReceiver {
    match receiver {
        Receiver::SelfKeyword => WrittenReceiver::SelfKeyword,
        Receiver::SelfName => WrittenReceiver::SelfName,
        Receiver::Other | Receiver::Sent => WrittenReceiver::Other,
    }
}

/// The argument list being gathered for the innermost call.
fn open_list(state: &mut RunState) -> &mut ArgumentList {
    state
        .arguments
        .last_mut()
        .expect("an argument list was begun")
}

/// The arguments and the block of the call `site` names, whose receiver is
/// in `base`.
fn call_inputs(
    site: &CallSite,
    base: usize,
    registers: &[Object],
    state: &mut RunState,
) -> (Vec<Object>, Option<Object>) {
    let arguments = if site.arguments_in_list {
        state
            .arguments
            .pop()
            .expect("an argument list was begun")
            .finish()
    } else {
        registers[base + 1..base + 1 + site.argument_count].to_vec()
    };
    let block = site
        .has_block
        .then(|| registers[base + site.block_offset()].clone());
    (arguments, block)
}

/// The prepared form of an assignment target, reading what was evaluated
/// ahead of the value from the registers.
fn prepared<'a>(target: &'a Target, registers: &[Object]) -> PreparedTarget<'a> {
    match target {
        Target::Named(named) => PreparedTarget::Named(named),
        Target::Setter {
            receiver,
            reader,
            safe,
            names_self,
            position,
        } => PreparedTarget::Receiver(ReceiverTarget::Setter {
            names_self: *names_self,
            receiver: registers[*receiver].clone(),
            reader,
            safe: *safe,
            position: *position,
        }),
        Target::Subscript {
            receiver,
            first,
            count,
            spread,
            sends,
            position,
        } => PreparedTarget::Receiver(ReceiverTarget::Subscript {
            receiver: registers[*receiver].clone(),
            subscripts: match registers[*first].view() {
                View::Array(elements) if *spread => elements.borrow().clone(),
                _ => registers[*first..*first + *count].to_vec(),
            },
            sends: *sends,
            position: *position,
        }),
        Target::ScopedConstant {
            namespace,
            name,
            position,
            ..
        } => PreparedTarget::Receiver(ReceiverTarget::ScopedConstant {
            namespace: registers[*namespace].clone(),
            name,
            position: *position,
        }),
        Target::Group(parts, position) => PreparedTarget::Group(
            parts.iter().map(|part| prepared(part, registers)).collect(),
            *position,
        ),
        Target::Splat(inner) => PreparedTarget::Splat(Box::new(prepared(inner, registers))),
    }
}

/// The operator an arithmetic or comparison instruction applies.
fn binary_operator(instruction: Instruction) -> BinaryOp {
    match instruction {
        Instruction::Add { .. } => BinaryOp::Add,
        Instruction::Subtract { .. } => BinaryOp::Subtract,
        Instruction::Multiply { .. } => BinaryOp::Multiply,
        Instruction::Divide { .. } => BinaryOp::Divide,
        Instruction::Modulo { .. } => BinaryOp::Modulo,
        Instruction::Equal { .. } => BinaryOp::Equal,
        Instruction::NotEqual { .. } => BinaryOp::NotEqual,
        Instruction::Less { .. } => BinaryOp::Less,
        Instruction::LessEqual { .. } => BinaryOp::LessEqual,
        Instruction::Greater { .. } => BinaryOp::Greater,
        Instruction::GreaterEqual { .. } => BinaryOp::GreaterEqual,
        other => unreachable!("{} applies no binary operator", other.name()),
    }
}
