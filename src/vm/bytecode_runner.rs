//! Running a compiled unit: one loop dispatching each instruction by
//! `match`, over a register file the unit's size. An instruction that needs
//! the interpreter, to call a method, read a constant, or raise, calls the
//! same `VirtualMachine` functions the tree walker does.

use crate::ast::{BinaryOp, Expression, UnaryOp};
use crate::bytecode::instruction::{Instruction, Operand, RANGE_EXCLUSIVE, RANGE_LITERAL_ENDS};
use crate::bytecode::unit::{CallSite, HashEntry, HashShape, Piece, Receiver, Unit};
use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::{Object, View};
use crate::vm::VirtualMachine;
use crate::vm::expression::{HashLiteral, Interpolation, WrittenPair};
use crate::vm::method_lookup::WrittenReceiver;
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
enum Next {
    Step,
    JumpTo(usize),
    Finish(Object),
}

/// The cells a unit's locals are held in, shared with the scope the unit
/// runs in, so `binding`, `local_variables`, and `eval` read and write them
/// too. A slot holds its cell from the unit's first write of the local on.
type LocalCells = Vec<Option<Rc<RefCell<Object>>>>;

impl VirtualMachine {
    /// Run `unit`, answering what its `Return` hands back.
    pub fn run_unit(&mut self, unit: &Unit) -> Result<Object, MetorexError> {
        let mut cells: LocalCells = vec![None; unit.locals.len()];
        // A continuation taken in the unit runs it again from the statement
        // its `callcc` was written in.
        self.run_restartable(|vm, statement| {
            let at = unit.statement_at(statement).unwrap_or(0);
            vm.run_unit_from(unit, &mut cells, at)
        })
    }

    fn run_unit_from(
        &mut self,
        unit: &Unit,
        cells: &mut LocalCells,
        start: usize,
    ) -> Result<Object, MetorexError> {
        let mut registers = vec![Object::Nil; unit.register_count];
        let mut at = start;
        let mut extension = Extension::default();
        loop {
            let instruction = unit.instructions[at];
            let position = unit.positions[at];
            if crate::vm::counters::enabled() {
                crate::vm::counters::count_instruction(instruction.name());
            }
            if let Instruction::Extend { high, wide } = instruction {
                extension = Extension {
                    high: usize::from(high),
                    wide: usize::from(wide),
                };
                at += 1;
                continue;
            }
            let ran = match unit.statement_starts[at] {
                Some(statement) => {
                    self.mark_statement(statement.index);
                    self.begin_statement(statement.position, true)
                        .and_then(|()| {
                            self.execute_instruction(
                                unit,
                                &mut registers,
                                cells,
                                instruction,
                                extension,
                                position,
                            )
                        })
                }
                None => self.execute_instruction(
                    unit,
                    &mut registers,
                    cells,
                    instruction,
                    extension,
                    position,
                ),
            };
            extension = Extension::default();
            match ran {
                Ok(Next::Step) => at += 1,
                Ok(Next::JumpTo(target)) => at = target,
                Ok(Next::Finish(value)) => return Ok(value),
                Err(error) => {
                    if let Some(spot) = &unit.error_spots[at] {
                        self.note_compiled_error_spot(&error, spot);
                    }
                    let Some(handler) = unit.handler_for(at) else {
                        return Err(error);
                    };
                    let Some((exception, raised_at)) =
                        crate::vm::exceptions::rescuable_exception(&error)
                    else {
                        return Err(error);
                    };
                    self.note_exception_location(
                        &exception,
                        &crate::vm::utils::position_to_location(raised_at),
                    );
                    self.set_current_exception(exception.clone());
                    registers[handler.register] = exception;
                    at = handler.target;
                }
            }
        }
    }

    fn execute_instruction(
        &mut self,
        unit: &Unit,
        registers: &mut [Object],
        cells: &mut LocalCells,
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
                let operator = binary_operator(instruction);
                registers[register(to)] =
                    self.operate_on_values(&operator, left, right, &position)?;
            }
            Instruction::Negate { to, from } => {
                let value = registers[register(from)].clone();
                registers[register(to)] =
                    self.apply_unary_operator(&UnaryOp::Minus, value, position)?;
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
            Instruction::Call { base, site } => {
                let site = &unit.call_sites[index(site)];
                let base = register(base);
                let receiver = registers[base].clone();
                let arguments = registers[base + 1..base + 1 + site.argument_count].to_vec();
                registers[base] = self.call_from_unit(receiver, site, arguments, position)?;
            }
            Instruction::GetLocal { to, slot } => {
                let slot = index(slot);
                registers[register(to)] = match &cells[slot] {
                    Some(cell) => cell.borrow().clone(),
                    None => self.read_identifier(&unit.locals[slot].text, position)?,
                };
            }
            Instruction::SetLocal { from, slot } => {
                let slot = index(slot);
                let value = registers[register(from)].clone();
                match &cells[slot] {
                    Some(cell) => *cell.borrow_mut() = value,
                    None => {
                        let name = unit.locals[slot].text.clone();
                        let target = Expression::Identifier {
                            name: name.clone(),
                            position,
                        };
                        self.assign_value(&target, value)?;
                        cells[slot] = self.environment().get_ref(&name);
                    }
                }
            }
            Instruction::GetIdentifier { to, name } => {
                let name = &unit.names[index(name)].text;
                registers[register(to)] = self.read_identifier(name, position)?;
            }
            Instruction::GetInstanceVariable { to, name } => {
                let name = &unit.names[index(name)].text;
                registers[register(to)] = self.eval_instance_var_read(name, position)?;
            }
            Instruction::SetInstanceVariable { from, name } => {
                let name = unit.names[index(name)].text.clone();
                let target = Expression::InstanceVariable { name, position };
                self.assign_value(&target, registers[register(from)].clone())?;
            }
            Instruction::GetConstant { to, name } => {
                let name = &unit.names[index(name)].text;
                registers[register(to)] = self.read_identifier(name, position)?;
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
            Instruction::Raise { from } => {
                let raised = registers[register(from)].clone();
                self.raise_exception(vec![raised], position)?;
            }
            Instruction::Return { from } => {
                return Ok(Next::Finish(registers[register(from)].clone()));
            }
        }
        Ok(Next::Step)
    }
}

impl VirtualMachine {
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

    /// Call the method a call site names, with what the tree walker does
    /// around a method call it evaluates.
    fn call_from_unit(
        &mut self,
        receiver: Object,
        site: &CallSite,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        crate::vm::program::enter_nesting(position)?;
        let attached = self.attached_block_flags.len();
        let written = match site.receiver {
            Receiver::SelfKeyword => WrittenReceiver::SelfKeyword,
            Receiver::SelfName => WrittenReceiver::SelfName,
            Receiver::Other => WrittenReceiver::Other,
        };
        let result = self
            .refuse_moved_receiver(&receiver, position)
            .and_then(|()| {
                self.call_evaluated_method(receiver, &site.name, arguments, written, position)
            });
        self.release_attached_blocks(attached);
        crate::vm::program::leave_nesting();
        self.note_failed_call(&result, &site.name, written.names_self(), position);
        result
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
