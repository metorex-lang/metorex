//! Compiling a body's statements to a `Unit`. A body holding a node the
//! compiler does not handle yet is refused with that node's type name, and
//! runs on the tree walker.

use crate::ast::{BinaryOp, Expression, InterpolationPart, Statement, UnaryOp};
use crate::bytecode::instruction::{
    BINARY_OPERATORS, FIRST_CONSTANT_OPERAND, Instruction, RANGE_EXCLUSIVE, RANGE_LITERAL_ENDS,
};
use crate::bytecode::unit::{
    CallSite, CaseArm, ErrorSpot, HashEntry, HashShape, MultipleAssignment, Name, Piece, Receiver,
    RegexLiteral, StartKind, Target, Unit,
};
use crate::lexer::Position;
use crate::object::Object;

/// The register the body's value is left in.
const ANSWER: usize = 0;

/// The registers an instruction can name without an `Extend`.
const REGISTER_LIMIT: usize = 256;

/// How many elements or pairs of a long Array or Hash literal are built at
/// a time.
const LITERAL_CHUNK: usize = 64;

/// What a register or table index past what an operand holds refuses with,
/// which the node being compiled is reported in place of.
const OVER_LIMIT: &str = "over limit";

/// Report `node` for a refusal over a limit.
fn naming(node: &'static str) -> impl Fn(&'static str) -> &'static str {
    move |refused| if refused == OVER_LIMIT { node } else { refused }
}

/// Compile `body`, or name the first node type it holds that the compiler
/// does not handle.
pub fn compile(body: &[Statement]) -> Result<Unit, &'static str> {
    compile_as(body, false)
}

/// Compile `body`, which is a method's body when `of_a_method`: its last
/// statement answers the method's value the way the tree walker answers it,
/// an `if` or `unless` there running as an `if` written as an expression.
pub fn compile_as(body: &[Statement], of_a_method: bool) -> Result<Unit, &'static str> {
    let mut compiler = Compiler {
        unit: Unit::new(),
        next_register: ANSWER + 1,
        spots: Vec::new(),
        assigned: crate::ast::collect_assigned_locals(body),
        loops: Vec::new(),
        pending: Pending::default(),
    };
    compiler.unit.use_registers(ANSWER + 1);
    for name in compiler.assigned.clone() {
        compiler.unit.local(&name);
    }
    let end = match body.last() {
        Some(last) => last.position(),
        None => Position::default(),
    };
    match body.split_last() {
        Some((last, before)) if of_a_method && answers_its_own_value(last) => {
            compiler.statements(before, ANSWER, true)?;
            let start = compiler.unit.instructions.len();
            compiler
                .terminal_statement(last)
                .map_err(naming(last.kind_name()))?;
            compiler.unit.start_statement(start, last.position());
            compiler.unit.set_start_kind(start, StartKind::Terminal);
        }
        _ => compiler.statements(body, ANSWER, true)?,
    }
    compiler.emit(Instruction::Return { from: ANSWER as u8 }, end);
    Ok(compiler.unit)
}

struct Compiler {
    unit: Unit,
    /// The first register no expression being compiled holds.
    next_register: usize,
    /// The error spot of each expression being compiled, the innermost last.
    spots: Vec<Option<ErrorSpot>>,
    /// The locals the body assigns, which the scope reserves on entry.
    assigned: Vec<String>,
    /// The loops being compiled, the innermost last.
    loops: Vec<Loop>,
    /// What the code compiled so far has opened and not yet closed.
    pending: Pending,
}

/// What a run has open at a place in the code: argument lists, call routes,
/// and scopes `case` arms opened. A jump out of the place drops them.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
struct Pending {
    lists: usize,
    routes: usize,
    scopes: usize,
}

/// A loop being compiled.
struct Loop {
    /// The register the loop answers in, which a `break` writes.
    to: usize,
    /// Where `redo` goes: the start of the body.
    redo_target: usize,
    /// The jumps `break` makes, to the end of the loop.
    breaks: Vec<usize>,
    /// The jumps `next` makes, to where the condition is read.
    nexts: Vec<usize>,
    /// What was open where the loop began.
    pending: Pending,
}

impl Compiler {
    /// Add an instruction, with the spot of the innermost expression being
    /// compiled that has one.
    fn emit(&mut self, instruction: Instruction, position: Position) -> usize {
        match instruction {
            Instruction::StartArguments => self.pending.lists += 1,
            Instruction::ArgumentsToArray { .. } => self.pending.lists -= 1,
            Instruction::ResolveCall { .. } | Instruction::ResolveScopedCall { .. } => {
                self.pending.routes += 1;
            }
            _ => {}
        }
        if let Instruction::Call { site, .. }
        | Instruction::CallResolved { site, .. }
        | Instruction::CallNativeName { site, .. }
        | Instruction::CallBlock { site, .. } = instruction
        {
            if self.unit.call_sites[usize::from(site)].arguments_in_list {
                self.pending.lists -= 1;
            }
            if matches!(instruction, Instruction::CallResolved { .. }) {
                self.pending.routes -= 1;
            }
        }
        let at = self.unit.emit(instruction, position);
        self.unit.error_spots[at] = self.spots.iter().rev().flatten().next().cloned();
        at
    }

    /// A register no expression being compiled holds.
    fn temporary(&mut self) -> Result<usize, &'static str> {
        let register = self.next_register;
        if register >= REGISTER_LIMIT {
            return Err(OVER_LIMIT);
        }
        self.next_register += 1;
        self.unit.use_registers(self.next_register);
        Ok(register)
    }

    /// Compile the last statement of a method body, which answers the
    /// method's value without passing through the statement runner: an
    /// assignment there is evaluated as an expression would be, and an `if`
    /// or `unless` runs as one written as an expression.
    fn terminal_statement(&mut self, statement: &Statement) -> Result<(), &'static str> {
        match statement {
            Statement::Assignment {
                target,
                value,
                position,
            } => self.assignment(target, value, ANSWER, *position),
            Statement::If {
                condition,
                then_branch,
                elsif_branches,
                else_branch,
                position,
            } => self.branches(
                condition,
                false,
                then_branch,
                elsif_branches,
                else_branch.as_deref(),
                ANSWER,
                *position,
                true,
            ),
            Statement::Unless {
                condition,
                then_branch,
                else_branch,
                position,
            } => self.branches(
                condition,
                true,
                then_branch,
                &[],
                else_branch.as_deref(),
                ANSWER,
                *position,
                true,
            ),
            other => self.statement(other, ANSWER),
        }
    }

    /// Compile a list of statements, leaving the value of the last in
    /// register `to`, or nil when there are none. `of_the_body` is set for
    /// the body's own statements, which a continuation can restart at.
    fn statements(
        &mut self,
        list: &[Statement],
        to: usize,
        of_the_body: bool,
    ) -> Result<(), &'static str> {
        self.statements_in(list, to, of_the_body, None)
    }

    /// Compile a list of statements as `statements` does. The statements of
    /// a branch of an `if` written as an expression say `enclosing_line`,
    /// the line the branch belongs to.
    fn statements_in(
        &mut self,
        list: &[Statement],
        to: usize,
        of_the_body: bool,
        enclosing_line: Option<usize>,
    ) -> Result<(), &'static str> {
        if list.is_empty() {
            let at = list.first().map(Statement::position).unwrap_or_default();
            self.emit(Instruction::LoadNil { to: to as u8 }, at);
        }
        for statement in list {
            let start = self.unit.instructions.len();
            self.statement(statement, to)
                .map_err(naming(statement.kind_name()))?;
            if of_the_body {
                self.unit.start_statement(start, statement.position());
            } else {
                let live = self.next_register;
                self.unit
                    .start_inner_statement(start, statement.position(), live);
            }
            if let (Some(enclosing_line), Statement::Expression { .. }) =
                (enclosing_line, statement)
            {
                self.unit
                    .set_start_kind(start, StartKind::InBranch { enclosing_line });
            }
        }
        Ok(())
    }

    fn statement(&mut self, statement: &Statement, to: usize) -> Result<(), &'static str> {
        let to_register = to as u8;
        match statement {
            Statement::Expression { expression, .. } => self.expression(expression, to),
            Statement::Assignment {
                target,
                value,
                position,
            } => {
                // An error the assignment raises with no spot of its own is
                // pointed at the setter or `[]=` it called.
                self.spots
                    .push(
                        crate::vm::error_spots::assignment_spot(target).map(|(start, name)| {
                            ErrorSpot {
                                start,
                                name: Some(name),
                            }
                        }),
                    );
                // Ruby's parser introduces the local where the assignment is
                // written, ahead of the value, so a block on the right-hand
                // side closes over the name being assigned. That is what
                // lets `walk = -> { walk.call }` call itself.
                if let Expression::Identifier { name, .. } = target
                    && self.assigned.contains(name)
                {
                    let slot = wide(Some(self.unit.local(name)))?;
                    self.emit(Instruction::Unhoist { slot }, *position);
                }
                let compiled = self.assignment(target, value, to, *position);
                self.spots.pop();
                compiled
            }
            Statement::MultipleAssignment {
                targets,
                values,
                position,
            } => self.multiple_assignment(targets, values, to, *position),
            Statement::DeclareLocals { names, position } => {
                for name in names {
                    let slot = wide(Some(self.unit.local(name)))?;
                    self.emit(Instruction::DeclareLocal { slot }, *position);
                }
                self.emit(Instruction::LoadNil { to: to_register }, *position);
                Ok(())
            }
            Statement::If {
                condition,
                then_branch,
                elsif_branches,
                else_branch,
                position,
            } => {
                self.branches(
                    condition,
                    false,
                    then_branch,
                    elsif_branches,
                    else_branch.as_deref(),
                    to,
                    *position,
                    false,
                )?;
                self.unhoist_assigned(statement)
            }
            Statement::Unless {
                condition,
                then_branch,
                else_branch,
                position,
            } => {
                self.branches(
                    condition,
                    true,
                    then_branch,
                    &[],
                    else_branch.as_deref(),
                    to,
                    *position,
                    false,
                )?;
                self.unhoist_assigned(statement)
            }
            Statement::While {
                condition,
                body,
                position,
            } => {
                self.while_loop(condition, body, to, *position)?;
                self.unhoist_assigned(statement)
            }
            Statement::DoWhile {
                condition,
                body,
                position,
            } => {
                self.do_while_loop(condition, body, to, *position)?;
                self.unhoist_assigned(statement)
            }
            Statement::For {
                variable,
                iterable,
                body,
                position,
            } => {
                // `for name in iterable` is a call of `each` with a block
                // that assigns `name`, which the tree walker runs the same
                // way.
                let looped = crate::ast::for_loop::for_over_each(
                    vec![Expression::Identifier {
                        name: variable.clone(),
                        position: *position,
                    }],
                    false,
                    iterable.clone(),
                    body.clone(),
                    *position,
                );
                // The desugared loop is a `begin` with no clauses around the
                // names the loop binds and the call of `each`, which runs its
                // statements in the scope holding the loop.
                let Statement::Expression {
                    expression: Expression::BeginRescue { body: looped, .. },
                    ..
                } = &looped
                else {
                    unreachable!("for_over_each answers a begin around the loop");
                };
                self.emit(Instruction::EnterLoop, *position);
                self.statements(looped, to, false)?;
                self.emit(Instruction::LeaveLoop, *position);
                self.unhoist_assigned(statement)
            }
            Statement::Break { value, position } => {
                let into = self.loops.last().map_or(to, |inner| inner.to);
                self.optional_value(value.as_deref(), into, *position)?;
                match self.loops.len() {
                    0 => {
                        self.emit(Instruction::ExitBreak { from: into as u8 }, *position);
                    }
                    _ => {
                        self.unwind_to_loop(*position);
                        let jump = self.emit(Instruction::Jump { target: 0 }, *position);
                        self.innermost_loop().breaks.push(jump);
                    }
                }
                Ok(())
            }
            Statement::Continue { value, position } => {
                self.optional_value(value.as_deref(), to, *position)?;
                match self.loops.len() {
                    0 => {
                        self.emit(Instruction::ExitNext { from: to_register }, *position);
                    }
                    _ => {
                        self.unwind_to_loop(*position);
                        let jump = self.emit(Instruction::Jump { target: 0 }, *position);
                        self.innermost_loop().nexts.push(jump);
                    }
                }
                Ok(())
            }
            Statement::Return { value, position } => {
                self.optional_value(value.as_ref(), to, *position)?;
                self.emit(
                    Instruction::ExitReturn {
                        from: to_register,
                        written: u8::from(value.is_some()),
                    },
                    *position,
                );
                Ok(())
            }
            // A block of statements runs in a scope of its own.
            Statement::Block {
                statements,
                position,
            } => {
                self.emit(Instruction::PushScope, *position);
                self.pending.scopes += 1;
                self.statements(statements, to, false)?;
                self.pending.scopes -= 1;
                self.emit(Instruction::PopScope, *position);
                Ok(())
            }
            Statement::Redo { position } => {
                match self.loops.last().map(|inner| inner.redo_target) {
                    None => {
                        self.emit(Instruction::ExitRedo, *position);
                    }
                    Some(target) => {
                        self.unwind_to_loop(*position);
                        let target = wide(Some(target))?;
                        self.emit(Instruction::Jump { target }, *position);
                    }
                }
                Ok(())
            }
            other => Err(other.kind_name()),
        }
    }

    /// The loop being compiled innermost.
    fn innermost_loop(&mut self) -> &mut Loop {
        self.loops.last_mut().expect("a loop is being compiled")
    }

    /// Before a jump out of the middle of the innermost loop's body, drop
    /// what was opened since the loop began.
    fn unwind_to_loop(&mut self, position: Position) {
        let at_loop = self.innermost_loop().pending;
        let now = self.pending;
        if now == at_loop {
            return;
        }
        self.emit(
            Instruction::Unwind {
                lists: (now.lists - at_loop.lists) as u8,
                routes: (now.routes - at_loop.routes) as u8,
                scopes: (now.scopes - at_loop.scopes) as u8,
            },
            position,
        );
        // The code after the jump is reached with what was open before it.
        self.pending = now;
    }

    /// Compile the value written after `break` or `next`, nil when none was.
    fn optional_value(
        &mut self,
        value: Option<&Expression>,
        to: usize,
        position: Position,
    ) -> Result<(), &'static str> {
        match value {
            Some(value) => self.expression(value, to),
            None => {
                self.emit(Instruction::LoadNil { to: to as u8 }, position);
                Ok(())
            }
        }
    }

    /// A local assigned inside a branch or a loop is a local from there on
    /// whether or not the assignment ran, so a block written later closes
    /// over it.
    fn unhoist_assigned(&mut self, statement: &Statement) -> Result<(), &'static str> {
        for name in crate::ast::collect_assigned_locals(std::slice::from_ref(statement)) {
            let slot = wide(Some(self.unit.local(&name)))?;
            self.emit(Instruction::Unhoist { slot }, statement.position());
        }
        Ok(())
    }

    /// Compile `if`, with its `elsif`s and `else`, or `unless` when
    /// `negated`, leaving the value of the branch taken in register `to`, or
    /// nil when none was.
    #[allow(clippy::too_many_arguments)]
    fn branches(
        &mut self,
        condition: &Expression,
        negated: bool,
        then_branch: &[Statement],
        elsif_branches: &[crate::ast::ElsifBranch],
        else_branch: Option<&[Statement]>,
        to: usize,
        position: Position,
        as_expression: bool,
    ) -> Result<(), &'static str> {
        let condition_line = as_expression.then_some(condition.position().line);
        let test = to as u8;
        let mut done = Vec::new();
        self.condition(condition, to)?;
        let skip = if negated {
            Instruction::JumpIfTrue { test, target: 0 }
        } else {
            Instruction::JumpIfFalse { test, target: 0 }
        };
        let mut not_taken = self.emit(skip, position);
        self.emit(Instruction::CoverBranch { arm: 0 }, position);
        self.statements_in(then_branch, to, false, condition_line)?;
        done.push(self.emit(Instruction::Jump { target: 0 }, position));
        for (index, elsif) in elsif_branches.iter().enumerate() {
            self.jump_here(not_taken)?;
            // Ruby counts an `elsif` line each time its condition is tested.
            self.emit(Instruction::CoverLine, elsif.position);
            self.condition(&elsif.condition, to)?;
            not_taken = self.emit(Instruction::JumpIfFalse { test, target: 0 }, position);
            let arm = u16::try_from(index + 1).map_err(|_| OVER_LIMIT)?;
            self.emit(Instruction::CoverBranch { arm }, position);
            let elsif_line = as_expression.then_some(elsif.position.line);
            self.statements_in(&elsif.body, to, false, elsif_line)?;
            done.push(self.emit(Instruction::Jump { target: 0 }, position));
        }
        self.jump_here(not_taken)?;
        let arm = u16::try_from(elsif_branches.len() + 1).map_err(|_| OVER_LIMIT)?;
        if negated {
            self.emit(Instruction::CoverBranch { arm: 1 }, position);
        } else {
            self.emit(Instruction::CoverBranch { arm }, position);
        }
        match else_branch {
            Some(else_branch) => self.statements_in(else_branch, to, false, condition_line)?,
            None => {
                self.emit(Instruction::LoadNil { to: test }, position);
            }
        }
        for jump in done {
            self.jump_here(jump)?;
        }
        Ok(())
    }

    /// Compile `while condition; body; end`, answering nil in register `to`,
    /// or the value a `break` carries.
    fn while_loop(
        &mut self,
        condition: &Expression,
        body: &[Statement],
        to: usize,
        position: Position,
    ) -> Result<(), &'static str> {
        let test = to as u8;
        self.emit(Instruction::EnterLoop, position);
        let condition_at = self.unit.instructions.len();
        // Each pass is a place the turn can be handed over, so a loop with
        // nothing in its body still lets the other threads run.
        self.emit(Instruction::Checkpoint, condition.position());
        self.condition(condition, to)?;
        let finished = self.emit(Instruction::JumpIfFalse { test, target: 0 }, position);
        self.emit(Instruction::CoverBranch { arm: 0 }, position);
        let body_at = self.unit.instructions.len();
        self.loops.push(Loop {
            to,
            redo_target: body_at,
            breaks: Vec::new(),
            nexts: Vec::new(),
            pending: self.pending,
        });
        let compiled = self.statements(body, to, false);
        let inner = self.loops.pop().expect("the loop pushed above");
        compiled?;
        let back = wide(Some(condition_at))?;
        self.emit(Instruction::Jump { target: back }, position);
        for jump in inner.nexts {
            self.jump_to(jump, condition_at)?;
        }
        self.jump_here(finished)?;
        self.emit(Instruction::LoadNil { to: test }, position);
        for jump in inner.breaks {
            self.jump_here(jump)?;
        }
        self.emit(Instruction::LeaveLoop, position);
        Ok(())
    }

    /// Compile `begin; body; end while condition`, which runs its body
    /// before it first reads the condition.
    fn do_while_loop(
        &mut self,
        condition: &Expression,
        body: &[Statement],
        to: usize,
        position: Position,
    ) -> Result<(), &'static str> {
        let test = to as u8;
        self.emit(Instruction::EnterLoop, position);
        let top = self.unit.instructions.len();
        self.emit(Instruction::CoverBranch { arm: 0 }, position);
        let body_at = self.unit.instructions.len();
        self.loops.push(Loop {
            to,
            redo_target: body_at,
            breaks: Vec::new(),
            nexts: Vec::new(),
            pending: self.pending,
        });
        let compiled = self.statements(body, to, false);
        let inner = self.loops.pop().expect("the loop pushed above");
        compiled?;
        for jump in inner.nexts {
            self.jump_here(jump)?;
        }
        self.emit(Instruction::Checkpoint, condition.position());
        self.condition(condition, to)?;
        let again = wide(Some(top))?;
        self.emit(
            Instruction::JumpIfTrue {
                test,
                target: again,
            },
            position,
        );
        self.emit(Instruction::LoadNil { to: test }, position);
        for jump in inner.breaks {
            self.jump_here(jump)?;
        }
        self.emit(Instruction::LeaveLoop, position);
        Ok(())
    }

    /// Compile an `if` or loop condition into register `to`, which the jump
    /// after it reads. `a and b` reads each side as a condition of its own,
    /// a Regexp on its own matches the last line read, and a Range written
    /// as a condition is a flip-flop.
    fn condition(&mut self, condition: &Expression, to: usize) -> Result<(), &'static str> {
        let test = to as u8;
        match condition {
            Expression::BinaryOp {
                op: op @ (BinaryOp::And | BinaryOp::Or),
                left,
                right,
                position,
            } => {
                self.condition(left, to)?;
                let decided = match op {
                    BinaryOp::Or => Instruction::JumpIfTrue { test, target: 0 },
                    _ => Instruction::JumpIfFalse { test, target: 0 },
                };
                let decided = self.emit(decided, *position);
                self.condition(right, to)?;
                self.jump_here(decided)
            }
            Expression::RegexLiteral { position, .. } => {
                let base = self.temporary()?;
                self.expression(condition, base)?;
                self.emit(Instruction::LastLine { to: base as u8 + 1 }, *position);
                self.temporary()?;
                let site = CallSite::new("=~", 1, Receiver::Sent);
                let site = wide(Some(self.unit.call_site(site)))?;
                self.emit(
                    Instruction::Call {
                        base: base as u8,
                        site,
                    },
                    *position,
                );
                self.emit(
                    Instruction::Move {
                        to: test,
                        from: base as u8,
                    },
                    *position,
                );
                self.next_register = base;
                Ok(())
            }
            Expression::Range {
                start,
                end,
                exclusive,
                position,
            } => self.flip_flop(start, end, *exclusive, to, *position),
            other => self.expression(other, to),
        }
    }

    /// Compile a flip-flop, `start..end` written as a condition: on from
    /// when `start` holds until when `end` holds.
    fn flip_flop(
        &mut self,
        start: &Expression,
        end: &Expression,
        exclusive: bool,
        to: usize,
        position: Position,
    ) -> Result<(), &'static str> {
        let test = to as u8;
        let integer_sides = [start, end]
            .into_iter()
            .filter(|side| matches!(side, Expression::IntLiteral { .. }))
            .count();
        let flip = wide(Some(self.unit.flip_flop(position, integer_sides)))?;
        self.emit(Instruction::FlipFlopState { to: test, flip }, position);
        let running = self.emit(Instruction::JumpIfTrue { test, target: 0 }, position);
        self.flip_flop_side(start, to, position)?;
        let off = self.emit(Instruction::JumpIfFalse { test, target: 0 }, position);
        self.emit(Instruction::FlipFlopSet { on: 1, flip }, position);
        let mut on = Vec::new();
        // `..` reads its second side the moment the first one holds, so a
        // range whose sides hold together is on for that one turn alone.
        // `...` waits for the next turn.
        if !exclusive {
            self.flip_flop_side(end, to, position)?;
            on.push(self.emit(Instruction::JumpIfFalse { test, target: 0 }, position));
            self.emit(Instruction::FlipFlopSet { on: 0, flip }, position);
        }
        on.push(self.emit(Instruction::Jump { target: 0 }, position));
        self.jump_here(running)?;
        self.flip_flop_side(end, to, position)?;
        on.push(self.emit(Instruction::JumpIfFalse { test, target: 0 }, position));
        self.emit(Instruction::FlipFlopSet { on: 0, flip }, position);
        on.push(self.emit(Instruction::Jump { target: 0 }, position));
        self.jump_here(off)?;
        self.emit(Instruction::LoadFalse { to: test }, position);
        let done = self.emit(Instruction::Jump { target: 0 }, position);
        for jump in on {
            self.jump_here(jump)?;
        }
        self.emit(Instruction::LoadTrue { to: test }, position);
        self.jump_here(done)
    }

    /// One side of a flip-flop. A number stands for the line the last read
    /// left off at, which is what `$.` reports.
    fn flip_flop_side(
        &mut self,
        side: &Expression,
        to: usize,
        position: Position,
    ) -> Result<(), &'static str> {
        let Expression::IntLiteral { value, .. } = side else {
            return self.expression(side, to);
        };
        let constant = self.constant(Object::Int(*value))?;
        self.emit(
            Instruction::LineIs {
                to: to as u8,
                constant,
            },
            position,
        );
        Ok(())
    }

    /// Compile `case subject; when ...; else ...; end` written as an
    /// expression, leaving the value of the arm taken in register `to`.
    fn case_expression(
        &mut self,
        subject: &Expression,
        cases: &[crate::ast::node::ExprMatchCase],
        else_case: Option<&Expression>,
        to: usize,
        position: Position,
    ) -> Result<(), &'static str> {
        let held = self.temporary()?;
        let matched = self.temporary()?;
        self.expression(subject, held)?;
        let mut done = Vec::new();
        for (index, case) in cases.iter().enumerate() {
            let arm = wide(Some(
                self.unit.case_arm(CaseArm {
                    pattern: case.pattern.clone(),
                    guard: case.guard.clone(),
                    position: case.position,
                    case_position: position,
                    index,
                    counted_line: (!matches!(case.body, Expression::BeginRescue { .. }))
                        .then(|| case.body.position().line),
                }),
            ))?;
            self.emit(
                Instruction::CaseArm {
                    subject: held as u8,
                    arm,
                },
                case.position,
            );
            let next_arm = self.emit(
                Instruction::JumpIfFalse {
                    test: matched as u8,
                    target: 0,
                },
                case.position,
            );
            self.pending.scopes += 1;
            self.expression(&case.body, to)?;
            self.pending.scopes -= 1;
            self.emit(Instruction::PopScope, case.position);
            done.push(self.emit(Instruction::Jump { target: 0 }, case.position));
            self.jump_here(next_arm)?;
        }
        let arm = u16::try_from(cases.len()).map_err(|_| OVER_LIMIT)?;
        self.emit(Instruction::CoverBranch { arm }, position);
        match else_case {
            Some(else_case) => {
                if !matches!(else_case, Expression::BeginRescue { .. }) {
                    self.emit(Instruction::CoverLine, else_case.position());
                }
                self.expression(else_case, to)?;
            }
            None => {
                self.emit(Instruction::LoadNil { to: to as u8 }, position);
            }
        }
        for jump in done {
            self.jump_here(jump)?;
        }
        Ok(())
    }

    /// Compile `target = value`, leaving what the assignment answers in
    /// register `to`.
    fn assignment(
        &mut self,
        target: &Expression,
        value: &Expression,
        to: usize,
        position: Position,
    ) -> Result<(), &'static str> {
        let held = self.next_register;
        let compiled = self.assignment_in_place(target, value, to, position);
        self.next_register = held;
        compiled
    }

    fn assignment_in_place(
        &mut self,
        target: &Expression,
        value: &Expression,
        to: usize,
        position: Position,
    ) -> Result<(), &'static str> {
        let from = to as u8;
        let named = match target {
            Expression::Identifier { name, .. } if self.assigned.contains(name) => {
                Some(Instruction::SetLocal {
                    from,
                    slot: wide(self.unit.locals.iter().position(|held| &held.text == name))?,
                })
            }
            Expression::Identifier { name, .. } if starts_uppercase(name) => {
                Some(Instruction::SetConstant {
                    from,
                    name: self.name(name)?,
                })
            }
            Expression::InstanceVariable { name, .. } => Some(Instruction::SetInstanceVariable {
                from,
                name: self.name(name)?,
            }),
            Expression::GlobalVariable { name, .. } => Some(Instruction::SetGlobal {
                from,
                name: self.name(name)?,
            }),
            Expression::ClassVariable { name, .. } => Some(Instruction::SetClassVariable {
                from,
                name: self.name(name)?,
            }),
            _ => None,
        };
        let shape = match named {
            Some(_) => None,
            None => Some(self.prepare(target)?),
        };
        if let Some(
            receiver_shape @ (Target::Setter { .. }
            | Target::Subscript { .. }
            | Target::ScopedConstant { .. }),
        ) = &shape
        {
            return self.assignment_on_receiver(
                receiver_shape.clone(),
                target,
                value,
                to,
                position,
            );
        }
        let decided = self.assigned_value(target, value, to)?;
        let instruction = match (named, shape) {
            (Some(instruction), _) => instruction,
            (None, Some(shape)) => Instruction::StoreTarget {
                from,
                target: wide(Some(self.unit.target(shape)))?,
            },
            (None, None) => unreachable!("a target is named or prepared"),
        };
        self.emit(instruction, position);
        if let Some(decided) = decided {
            self.jump_here(decided)?;
        }
        Ok(())
    }

    /// Compile the value of an assignment to a variable or bare constant,
    /// answering the jump that skips the write when there is one.
    /// `CONST ||= value` and `$global ||= value` read a target not defined
    /// yet as nil without the notice reading it would give, so only the
    /// right-hand side is evaluated then. `target ||= value` is `target ||
    /// target = value`, and `&&=` the same with `&&`, so the target is not
    /// written when what it reads decides the answer.
    fn assigned_value(
        &mut self,
        target: &Expression,
        value: &Expression,
        to: usize,
    ) -> Result<Option<usize>, &'static str> {
        let to_register = to as u8;
        if let Expression::BinaryOp {
            op: BinaryOp::Or,
            left,
            right,
            position,
        } = value
        {
            let same_global = matches!(
                (target, left.as_ref()),
                (
                    Expression::GlobalVariable { name, .. },
                    Expression::GlobalVariable { name: read, .. }
                ) if name == read
            );
            if crate::vm::statement::constant_target_matches(target, left) || same_global {
                let expression = wide(Some(self.unit.defined_expression(target.clone())))?;
                self.emit(
                    Instruction::IsDefined {
                        to: to_register,
                        expression,
                    },
                    *position,
                );
                let undefined = self.emit(
                    Instruction::JumpIfNil {
                        test: to_register,
                        target: 0,
                    },
                    *position,
                );
                let decided = self.logical_value(target, value, to)?;
                let done = self.emit(Instruction::Jump { target: 0 }, *position);
                self.jump_here(undefined)?;
                self.expression(right, to)?;
                self.jump_here(done)?;
                return Ok(decided);
            }
        }
        self.logical_value(target, value, to)
    }

    /// Compile `value`, and when it is the `target || value` or `target &&
    /// value` an operator assignment is written as, answer the jump taken
    /// when what the target reads decides the answer.
    fn logical_value(
        &mut self,
        target: &Expression,
        value: &Expression,
        to: usize,
    ) -> Result<Option<usize>, &'static str> {
        let Some((or_assigning, reading, right)) =
            crate::vm::statement::logical_assignment(target, value)
        else {
            self.expression(value, to)?;
            return Ok(None);
        };
        self.expression(reading, to)?;
        let test = to as u8;
        let jump = if or_assigning {
            Instruction::JumpIfTrue { test, target: 0 }
        } else {
            Instruction::JumpIfFalse { test, target: 0 }
        };
        let decided = self.emit(jump, value.position());
        self.expression(right, to)?;
        Ok(Some(decided))
    }

    /// Compile an assignment through a setter, `[]=`, or `Ns::Name =`, whose
    /// receiver `shape` was evaluated into registers already.
    fn assignment_on_receiver(
        &mut self,
        mut shape: Target,
        target: &Expression,
        value: &Expression,
        to: usize,
        position: Position,
    ) -> Result<(), &'static str> {
        let from = to as u8;
        // `held&.name = value` evaluates nothing more when there is no
        // receiver, and answers nil.
        let missing_receiver = match &shape {
            Target::Setter {
                receiver,
                safe: true,
                ..
            } => Some(self.emit(
                Instruction::JumpIfNil {
                    test: *receiver as u8,
                    target: 0,
                },
                position,
            )),
            _ => None,
        };
        let mut decided = None;
        match value {
            // The parser writes `target op= operand` as `target = target op
            // operand`, and the target is read and written through the one
            // evaluation of its receiver.
            Expression::BinaryOp {
                op,
                left,
                right,
                position: operator_at,
            } if left.as_ref() == target => {
                if let Target::ScopedConstant { or_assigning, .. } = &mut shape {
                    *or_assigning = matches!(op, BinaryOp::Or);
                }
                let read = wide(Some(self.unit.target(shape.clone())))?;
                match op {
                    // `||=` and `&&=` write nothing when the value they read
                    // already decides the answer.
                    BinaryOp::Or | BinaryOp::And => {
                        self.emit(
                            Instruction::ReadTarget {
                                to: from,
                                target: read,
                            },
                            *operator_at,
                        );
                        let jump = if matches!(op, BinaryOp::Or) {
                            Instruction::JumpIfTrue {
                                test: from,
                                target: 0,
                            }
                        } else {
                            Instruction::JumpIfFalse {
                                test: from,
                                target: 0,
                            }
                        };
                        decided = Some(self.emit(jump, *operator_at));
                        self.expression(right, to)?;
                    }
                    _ => {
                        let current = self.temporary()?;
                        let operand = self.temporary()?;
                        self.emit(
                            Instruction::ReadTarget {
                                to: current as u8,
                                target: read,
                            },
                            *operator_at,
                        );
                        self.expression(right, operand)?;
                        self.emit(
                            Instruction::BinaryOperator {
                                to: from,
                                first: current as u8,
                                op: operator_index(op),
                            },
                            *operator_at,
                        );
                    }
                }
            }
            // `held <<= 2` calls the shift the value defines.
            Expression::MethodCall {
                receiver,
                method,
                arguments,
                position: call_at,
                ..
            } if receiver.as_ref() == target => {
                let read = wide(Some(self.unit.target(shape.clone())))?;
                let base = self.temporary()?;
                self.emit(
                    Instruction::ReadTarget {
                        to: base as u8,
                        target: read,
                    },
                    *call_at,
                );
                self.plain_arguments(arguments)?;
                let site = CallSite::new(method, arguments.len(), Receiver::Sent);
                let site = wide(Some(self.unit.call_site(site)))?;
                self.emit(
                    Instruction::Call {
                        base: base as u8,
                        site,
                    },
                    *call_at,
                );
                self.emit(
                    Instruction::Move {
                        to: from,
                        from: base as u8,
                    },
                    *call_at,
                );
            }
            _ => self.expression(value, to)?,
        }
        let store = wide(Some(self.unit.target(shape)))?;
        self.emit(
            Instruction::StoreTarget {
                from,
                target: store,
            },
            position,
        );
        if let Some(decided) = decided {
            self.jump_here(decided)?;
        }
        if let Some(missing_receiver) = missing_receiver {
            let done = self.emit(Instruction::Jump { target: 0 }, position);
            self.jump_here(missing_receiver)?;
            self.emit(Instruction::LoadNil { to: from }, position);
            self.jump_here(done)?;
        }
        Ok(())
    }

    /// Evaluate what `target` needs ahead of its value into registers kept
    /// until the assignment is done: the receiver of a setter, the receiver
    /// and subscripts of `[]=`, and the namespace of `Ns::Name`. Ruby
    /// evaluates these left to right before the right-hand side.
    fn prepare(&mut self, target: &Expression) -> Result<Target, &'static str> {
        Ok(match target {
            // `(a, b)` names a group of targets, which takes its value
            // apart the way the whole list does.
            Expression::Array { elements, position } => {
                let mut parts = Vec::with_capacity(elements.len());
                for element in elements {
                    parts.push(self.prepare(element)?);
                }
                Target::Group(parts, *position)
            }
            Expression::Splat { expression, .. } => {
                Target::Splat(Box::new(self.prepare(expression)?))
            }
            Expression::Index {
                array,
                index,
                position,
            } => {
                let receiver = self.temporary()?;
                self.expression(array, receiver)?;
                // `held[*subscripts] = value` spreads the values across the
                // subscript, the same way `held.[]=(*subscripts, value)` does.
                if let Expression::Splat {
                    expression: spread,
                    position: splat_at,
                } = index.as_ref()
                {
                    let first = self.spread_subscripts(spread, *splat_at)?;
                    return Ok(Target::Subscript {
                        receiver,
                        first,
                        count: 1,
                        spread: true,
                        sends: true,
                        position: *position,
                    });
                }
                let first = self.temporary()?;
                self.expression(index, first)?;
                Target::Subscript {
                    receiver,
                    first,
                    count: 1,
                    spread: false,
                    sends: false,
                    position: *position,
                }
            }
            Expression::MethodCall {
                receiver: receiver_expression,
                method,
                arguments,
                position,
                ..
            } => {
                let receiver = self.temporary()?;
                self.expression(receiver_expression, receiver)?;
                match (method.as_str(), arguments.as_slice()) {
                    // `values[start, length] = other` parses as a call to
                    // `[]`, and assigning to it sends `[]=` with the same
                    // subscripts.
                    ("[]", _) if arguments.iter().any(gathered_in_a_list) => {
                        let first = self.temporary()?;
                        let mut site = self.call_arguments("[]", arguments, None)?;
                        site.arguments_in_list = true;
                        self.emit(Instruction::ArgumentsToArray { to: first as u8 }, *position);
                        self.next_register = first + 1;
                        Target::Subscript {
                            receiver,
                            first,
                            count: 1,
                            spread: true,
                            sends: true,
                            position: *position,
                        }
                    }
                    ("[]", _) => {
                        let first = self.next_register;
                        self.plain_arguments(arguments)?;
                        Target::Subscript {
                            receiver,
                            first,
                            count: arguments.len(),
                            spread: false,
                            sends: true,
                            position: *position,
                        }
                    }
                    (crate::parser::SAFE_CALL, [Expression::Symbol { value: named, .. }]) => {
                        Target::Setter {
                            receiver,
                            reader: named.clone(),
                            safe: true,
                            names_self: receiver_written(receiver_expression) != Receiver::Other,
                            position: *position,
                        }
                    }
                    _ => Target::Setter {
                        receiver,
                        reader: method.clone(),
                        safe: false,
                        names_self: receiver_written(receiver_expression) != Receiver::Other,
                        position: *position,
                    },
                }
            }
            Expression::ScopeResolution {
                namespace,
                name,
                position,
            } => {
                let register = self.temporary()?;
                self.expression(namespace, register)?;
                Target::ScopedConstant {
                    namespace: register,
                    name: name.clone(),
                    or_assigning: false,
                    position: *position,
                }
            }
            named => Target::Named(named.clone()),
        })
    }

    /// Compile `targets = values`, which spreads the values across the
    /// targets and answers the right-hand side as written.
    fn multiple_assignment(
        &mut self,
        targets: &[Expression],
        values: &[Expression],
        to: usize,
        position: Position,
    ) -> Result<(), &'static str> {
        let held = self.next_register;
        let mut prepared = Vec::with_capacity(targets.len());
        for target in targets {
            prepared.push(self.prepare(target)?);
        }
        let lone_value = match values {
            [single] if !matches!(single, Expression::Splat { .. }) => {
                self.expression(single, to)?;
                true
            }
            _ => {
                let gathered = Expression::Array {
                    elements: values.to_vec(),
                    position,
                };
                self.expression(&gathered, to)?;
                false
            }
        };
        let shape = wide(Some(self.unit.multiple_assignment(MultipleAssignment {
            targets: prepared,
            lone_value,
            position,
        })))?;
        self.emit(
            Instruction::AssignMultiple {
                from: to as u8,
                shape,
            },
            position,
        );
        self.next_register = held;
        Ok(())
    }

    /// Compile a call's arguments to consecutive registers from the next
    /// free one, refusing the forms the compiler does not handle yet.
    fn plain_arguments(&mut self, arguments: &[Expression]) -> Result<(), &'static str> {
        for argument in arguments {
            if matches!(
                argument,
                Expression::Splat { .. }
                    | Expression::KeywordSplat { .. }
                    | Expression::BlockArg { .. }
            ) {
                return Err(argument.kind_name());
            }
            let register = self.temporary()?;
            self.expression(argument, register)?;
        }
        Ok(())
    }

    /// Point the jump at `at` to the next instruction.
    fn jump_here(&mut self, at: usize) -> Result<(), &'static str> {
        let here = self.unit.instructions.len();
        self.jump_to(at, here)
    }

    /// Point the jump at `at` to the instruction at `target`.
    fn jump_to(&mut self, at: usize, target: usize) -> Result<(), &'static str> {
        let here = wide(Some(target))?;
        self.unit.instructions[at] = match self.unit.instructions[at] {
            Instruction::Jump { .. } => Instruction::Jump { target: here },
            Instruction::JumpIfTrue { test, .. } => Instruction::JumpIfTrue { test, target: here },
            Instruction::JumpIfFalse { test, .. } => {
                Instruction::JumpIfFalse { test, target: here }
            }
            Instruction::JumpIfNil { test, .. } => Instruction::JumpIfNil { test, target: here },
            Instruction::SafeNavigation { test, .. } => {
                Instruction::SafeNavigation { test, target: here }
            }
            other => unreachable!("{} is not a jump", other.name()),
        };
        Ok(())
    }

    /// Compile `expression` to leave its value in register `to`.
    fn expression(&mut self, expression: &Expression, to: usize) -> Result<(), &'static str> {
        self.spots.push(
            crate::vm::error_spots::spot_of(expression)
                .map(|(start, name)| ErrorSpot { start, name }),
        );
        let held = self.next_register;
        let compiled = self
            .expression_in_place(expression, to)
            .map_err(naming(expression.kind_name()));
        self.next_register = held;
        self.spots.pop();
        compiled
    }

    fn expression_in_place(
        &mut self,
        expression: &Expression,
        to: usize,
    ) -> Result<(), &'static str> {
        let position = expression.position();
        let to_register = to as u8;
        let instruction = match expression {
            Expression::IntLiteral { value, .. } => self.integer(*value, to_register)?,
            Expression::MagicLine { position } => {
                self.integer(position.line as i64, to_register)?
            }
            Expression::BigIntLiteral { digits, .. } => {
                let Some(value) = num_bigint::BigInt::parse_bytes(digits.as_bytes(), 10) else {
                    return Err(expression.kind_name());
                };
                Instruction::LoadConstant {
                    to: to_register,
                    constant: self.constant(Object::integer(value))?,
                }
            }
            Expression::RegexLiteral { pattern, flags, .. } => Instruction::LoadRegex {
                to: to_register,
                pattern: wide(Some(self.unit.regex(RegexLiteral {
                    pattern: pattern.clone(),
                    flags: flags.clone(),
                })))?,
            },
            Expression::MagicFile { .. } => Instruction::LoadFile { to: to_register },
            Expression::MagicDir { .. } => Instruction::LoadDirectory { to: to_register },
            Expression::ClassVariable { name, .. } => Instruction::GetClassVariable {
                to: to_register,
                name: self.name(name)?,
            },
            Expression::TopLevelConstant { name, .. } => Instruction::GetTopLevelConstant {
                to: to_register,
                name: self.name(name)?,
            },
            Expression::ScopeResolution {
                namespace, name, ..
            } => {
                self.expression(namespace, to)?;
                Instruction::GetScopedConstant {
                    namespace: to_register,
                    name: self.name(name)?,
                }
            }
            Expression::Range {
                start,
                end,
                exclusive,
                ..
            } => {
                let first = self.temporary()?;
                self.expression(start, first)?;
                let last = self.temporary()?;
                self.expression(end, last)?;
                let mut flags = 0;
                if *exclusive {
                    flags |= RANGE_EXCLUSIVE;
                }
                if crate::vm::eval::literal_range_end(start)
                    && crate::vm::eval::literal_range_end(end)
                {
                    flags |= RANGE_LITERAL_ENDS;
                }
                Instruction::NewRange {
                    to: to_register,
                    first: first as u8,
                    flags,
                }
            }
            Expression::If {
                condition,
                then_branch,
                elsif_branches,
                else_branch,
                position,
            } => {
                return self.branches(
                    condition,
                    false,
                    then_branch,
                    elsif_branches,
                    else_branch.as_deref(),
                    to,
                    *position,
                    true,
                );
            }
            Expression::Unless {
                condition,
                then_branch,
                else_branch,
                position,
            } => {
                return self.branches(
                    condition,
                    true,
                    then_branch,
                    &[],
                    else_branch.as_deref(),
                    to,
                    *position,
                    true,
                );
            }
            Expression::Case {
                expression: subject,
                cases,
                else_case,
                position,
            } => {
                return self.case_expression(subject, cases, else_case.as_deref(), to, *position);
            }
            Expression::Defined { expression, .. } => Instruction::IsDefined {
                to: to_register,
                expression: wide(Some(self.unit.defined_expression(*expression.clone())))?,
            },
            Expression::Index { array, index, .. } => {
                let collection = self.temporary()?;
                self.expression(array, collection)?;
                // `held[*subscripts]` spreads the values across the
                // subscript, the same way `held.[](*subscripts)` does.
                if let Expression::Splat {
                    expression: spread,
                    position: splat_at,
                } = index.as_ref()
                {
                    let subscripts = self.spread_subscripts(spread, *splat_at)?;
                    self.emit(
                        Instruction::IndexSpread {
                            to: to_register,
                            collection: collection as u8,
                            subscripts: subscripts as u8,
                        },
                        position,
                    );
                    return Ok(());
                }
                let key = self.temporary()?;
                self.expression(index, key)?;
                Instruction::Index {
                    to: to_register,
                    collection: collection as u8,
                    key: key as u8,
                }
            }
            Expression::Dictionary { entries, .. } => {
                let entries = crate::vm::expression::flattened_entries(entries);
                let mut chunks = entries.chunks(LITERAL_CHUNK);
                let shape = self.hash_shape(chunks.next().unwrap_or(&[]))?;
                let shape = wide(Some(self.unit.hash(shape)))?;
                self.emit(
                    Instruction::NewHash {
                        to: to_register,
                        shape,
                    },
                    position,
                );
                for chunk in chunks {
                    let shape = self.hash_shape(chunk)?;
                    let shape = wide(Some(self.unit.hash(shape)))?;
                    self.emit(
                        Instruction::AddToHash {
                            hash: to_register,
                            shape,
                        },
                        position,
                    );
                }
                return Ok(());
            }
            Expression::InterpolatedString { parts, .. } => {
                let pieces = self.pieces(parts)?;
                Instruction::Interpolate {
                    to: to_register,
                    shape: wide(Some(self.unit.interpolation(pieces)))?,
                }
            }
            Expression::FloatLiteral { value, .. } => Instruction::LoadConstant {
                to: to_register,
                constant: self.constant(Object::Float(*value))?,
            },
            Expression::BoolLiteral { value: true, .. } => {
                Instruction::LoadTrue { to: to_register }
            }
            Expression::BoolLiteral { value: false, .. } => {
                Instruction::LoadFalse { to: to_register }
            }
            Expression::NilLiteral { .. } => Instruction::LoadNil { to: to_register },
            Expression::SelfExpr { .. } => Instruction::LoadSelf { to: to_register },
            Expression::StringLiteral { value, .. } => Instruction::NewString {
                to: to_register,
                text: self.name(value)?,
            },
            Expression::Symbol { value, .. } => Instruction::LoadSymbol {
                to: to_register,
                name: self.name(value)?,
            },
            Expression::Identifier { name, .. } => self.identifier(name, to_register)?,
            Expression::InstanceVariable { name, .. } => Instruction::GetInstanceVariable {
                to: to_register,
                name: self.name(name)?,
            },
            Expression::GlobalVariable { name, .. } => Instruction::GetGlobal {
                to: to_register,
                name: self.name(name)?,
            },
            Expression::Grouped { expression, .. } => return self.expression(expression, to),
            Expression::BinaryOp {
                op: BinaryOp::Assign,
                left,
                right,
                ..
            } => return self.assignment(left, right, to, position),
            Expression::BinaryOp {
                op: op @ (BinaryOp::And | BinaryOp::Or),
                left,
                right,
                ..
            } => {
                self.expression(left, to)?;
                let jump = match op {
                    BinaryOp::And => Instruction::JumpIfFalse {
                        test: to_register,
                        target: 0,
                    },
                    _ => Instruction::JumpIfTrue {
                        test: to_register,
                        target: 0,
                    },
                };
                let decided = self.emit(jump, position);
                self.expression(right, to)?;
                return self.jump_here(decided);
            }
            Expression::BinaryOp {
                op, left, right, ..
            } => {
                let Some(make) = binary_instruction(op) else {
                    let first = self.temporary()?;
                    self.expression(left, first)?;
                    let second = self.temporary()?;
                    self.expression(right, second)?;
                    self.emit(
                        Instruction::BinaryOperator {
                            to: to_register,
                            first: u8::try_from(first).map_err(|_| OVER_LIMIT)?,
                            op: operator_index(op),
                        },
                        position,
                    );
                    return Ok(());
                };
                let left_register = self.operand_temporary(expression)?;
                self.expression(left, left_register)?;
                let right_register = self.operand_temporary(expression)?;
                self.expression(right, right_register)?;
                make(to_register, left_register as u8, right_register as u8)
            }
            Expression::UnaryOp { op, operand, .. } => {
                self.expression(operand, to)?;
                match op {
                    UnaryOp::Minus => Instruction::Negate {
                        to: to_register,
                        from: to_register,
                    },
                    UnaryOp::Not => Instruction::Not {
                        to: to_register,
                        from: to_register,
                    },
                    UnaryOp::Plus => Instruction::Positive {
                        to: to_register,
                        from: to_register,
                    },
                }
            }
            Expression::Array { elements, .. }
                if elements
                    .iter()
                    .any(|element| matches!(element, Expression::Splat { .. })) =>
            {
                // `[first, *rest]` splices what a splat spreads in place.
                self.emit(
                    Instruction::NewArray {
                        to: to_register,
                        first: 0,
                        count: 0,
                    },
                    position,
                );
                for element in elements {
                    let register = self.temporary()?;
                    let instruction = match element {
                        Expression::Splat { expression, .. } => {
                            self.expression(expression, register)?;
                            Instruction::SplatInto {
                                array: to_register,
                                from: register as u8,
                            }
                        }
                        _ => {
                            self.expression(element, register)?;
                            Instruction::AppendArray {
                                array: to_register,
                                first: register as u8,
                                count: 1,
                            }
                        }
                    };
                    self.emit(instruction, element.position());
                    self.next_register = register;
                }
                return Ok(());
            }
            // Outside of an argument list, a splat makes a new Array of what
            // it spreads, the way `[*held]` does.
            Expression::Splat {
                expression: spread, ..
            } => {
                self.emit(
                    Instruction::NewArray {
                        to: to_register,
                        first: 0,
                        count: 0,
                    },
                    position,
                );
                let register = self.temporary()?;
                self.expression(spread, register)?;
                Instruction::SplatInto {
                    array: to_register,
                    from: register as u8,
                }
            }
            // Outside of an argument list, `**expr` and `&expr` are `expr`.
            Expression::KeywordSplat {
                expression: inner, ..
            }
            | Expression::BlockArg {
                expression: inner, ..
            } => return self.expression(inner, to),
            Expression::Yield { arguments, .. } => {
                // The block is looked for before the arguments are evaluated,
                // so a method called without one raises before they are.
                let base = self.temporary()?;
                self.emit(Instruction::FindBlock { to: base as u8 }, position);
                let site = self.call_arguments("yield", arguments, None)?;
                let site = wide(Some(self.unit.call_site(site)))?;
                self.emit(
                    Instruction::CallBlock {
                        base: base as u8,
                        site,
                    },
                    position,
                );
                Instruction::Move {
                    to: to_register,
                    from: base as u8,
                }
            }
            Expression::Lambda { .. } => Instruction::MakeBlock {
                to: to_register,
                literal: wide(Some(self.unit.block(expression.clone())))?,
            },
            Expression::PatternTest {
                value,
                pattern,
                refuses,
                ..
            } => {
                self.expression(value, to)?;
                Instruction::MatchPattern {
                    register: to_register,
                    pattern: wide(Some(self.unit.pattern((**pattern).clone(), *refuses)))?,
                }
            }
            Expression::Call {
                callee,
                arguments,
                trailing_block,
                ..
            } => return self.bare_call(callee, arguments, trailing_block.as_deref(), to, position),
            Expression::Array { elements, .. } => {
                // A long literal is built a chunk of elements at a time, so
                // its elements need no more registers than a chunk does.
                let mut chunks = elements.chunks(LITERAL_CHUNK);
                let first_chunk = chunks.next().unwrap_or(&[]);
                let first = self.chunk_of_elements(first_chunk)?;
                self.emit(
                    Instruction::NewArray {
                        to: to_register,
                        first,
                        count: first_chunk.len() as u8,
                    },
                    position,
                );
                for chunk in chunks {
                    let first = self.chunk_of_elements(chunk)?;
                    self.emit(
                        Instruction::AppendArray {
                            array: to_register,
                            first,
                            count: chunk.len() as u8,
                        },
                        position,
                    );
                }
                return Ok(());
            }
            Expression::MethodCall {
                receiver,
                method,
                arguments,
                trailing_block,
                ..
            } => {
                return self.method_call(
                    receiver,
                    method,
                    arguments,
                    trailing_block.as_deref(),
                    to,
                    position,
                );
            }
            other => return Err(other.kind_name()),
        };
        self.emit(instruction, position);
        Ok(())
    }

    /// Compile `receiver.method(arguments) { block }`, leaving the answer in
    /// register `to`.
    fn method_call(
        &mut self,
        receiver: &Expression,
        method: &str,
        arguments: &[Expression],
        trailing_block: Option<&Expression>,
        to: usize,
        position: Position,
    ) -> Result<(), &'static str> {
        let to_register = to as u8;
        // `native_fn[args]` calls the Kernel function with the bracketed
        // arguments in an Array, which is `private [:a, :b]`. The name is
        // looked up before the receiver is evaluated, since evaluating it
        // would run the function.
        let mut native_done = None;
        if method == "[]"
            && let Expression::Identifier { name, .. } = receiver
        {
            let base = self.temporary()?;
            let name_index = self.name(name)?;
            self.emit(
                Instruction::IsNativeName {
                    to: base as u8,
                    name: name_index,
                },
                position,
            );
            let not_native = self.emit(
                Instruction::JumpIfFalse {
                    test: base as u8,
                    target: 0,
                },
                position,
            );
            let mut site = self.call_arguments(name, arguments, None)?;
            site.receiver = Receiver::SelfKeyword;
            let site = wide(Some(self.unit.call_site(site)))?;
            self.emit(
                Instruction::CallNativeName {
                    base: base as u8,
                    site,
                },
                position,
            );
            self.emit(
                Instruction::Move {
                    to: to_register,
                    from: base as u8,
                },
                position,
            );
            native_done = Some(self.emit(Instruction::Jump { target: 0 }, position));
            self.jump_here(not_native)?;
            self.next_register = base;
        }
        let base = self.temporary()?;
        self.expression(receiver, base)?;
        // `held&.name(args) { block }` runs none of what follows when there
        // is no receiver, so the arguments are never evaluated.
        let missing_receiver = (method == crate::parser::SAFE_CALL).then(|| {
            self.emit(
                Instruction::SafeNavigation {
                    test: base as u8,
                    target: 0,
                },
                position,
            )
        });
        let mut site = self.call_arguments(method, arguments, trailing_block)?;
        site.receiver = receiver_written(receiver);
        let site = wide(Some(self.unit.call_site(site)))?;
        self.emit(
            Instruction::Call {
                base: base as u8,
                site,
            },
            position,
        );
        if base != to {
            self.emit(
                Instruction::Move {
                    to: to_register,
                    from: base as u8,
                },
                position,
            );
        }
        if let Some(missing_receiver) = missing_receiver {
            let done = self.emit(Instruction::Jump { target: 0 }, position);
            self.jump_here(missing_receiver)?;
            self.emit(Instruction::LoadNil { to: to_register }, position);
            self.jump_here(done)?;
        }
        if let Some(native_done) = native_done {
            self.jump_here(native_done)?;
        }
        Ok(())
    }

    /// Compile `name(arguments) { block }`, written without a receiver, or
    /// `Namespace::name(arguments)`, leaving the answer in register `to`.
    fn bare_call(
        &mut self,
        callee: &Expression,
        arguments: &[Expression],
        trailing_block: Option<&Expression>,
        to: usize,
        position: Position,
    ) -> Result<(), &'static str> {
        let (name, namespace) = match callee {
            Expression::Identifier { name, .. } => (name, None),
            Expression::ScopeResolution {
                namespace, name, ..
            } => (name, Some(namespace.as_ref())),
            other => return Err(other.kind_name()),
        };
        let resolve_at = match namespace {
            None => self.emit(Instruction::ResolveCall { site: 0 }, position),
            Some(namespace) => {
                let register = self.temporary()?;
                self.expression(namespace, register)?;
                self.emit(
                    Instruction::ResolveScopedCall {
                        namespace: register as u8,
                        site: 0,
                    },
                    position,
                )
            }
        };
        let base = self.temporary()?;
        let mut site = self.call_arguments(name, arguments, trailing_block)?;
        site.receiver = match namespace {
            None => Receiver::SelfKeyword,
            Some(namespace) => receiver_written(namespace),
        };
        let site = wide(Some(self.unit.call_site(site)))?;
        self.unit.instructions[resolve_at] = match self.unit.instructions[resolve_at] {
            Instruction::ResolveScopedCall { namespace, .. } => {
                Instruction::ResolveScopedCall { namespace, site }
            }
            _ => Instruction::ResolveCall { site },
        };
        self.emit(
            Instruction::CallResolved {
                base: base as u8,
                site,
            },
            position,
        );
        self.emit(
            Instruction::Move {
                to: to as u8,
                from: base as u8,
            },
            position,
        );
        Ok(())
    }

    /// Compile a call's arguments and block to the registers after the
    /// receiver's, which is the register before the next free one, and
    /// answer the call site for them. Arguments with `*`, `**`, or `&` among
    /// them are gathered in an argument list instead.
    fn call_arguments(
        &mut self,
        name: &str,
        arguments: &[Expression],
        trailing_block: Option<&Expression>,
    ) -> Result<CallSite, &'static str> {
        let in_list = arguments.iter().any(gathered_in_a_list);
        let mut site = CallSite::new(name, 0, Receiver::Other);
        site.written_with_arguments = !arguments.is_empty();
        site.keyword_splat = arguments
            .iter()
            .any(|argument| matches!(argument, Expression::KeywordSplat { .. }));
        site.has_block = trailing_block.is_some();
        if !in_list {
            site.argument_count = arguments.len();
            self.plain_arguments(arguments)?;
            if let Some(block) = trailing_block {
                let register = self.temporary()?;
                self.expression(block, register)?;
            }
            return Ok(site);
        }
        site.arguments_in_list = true;
        let block_register = match trailing_block {
            Some(_) => Some(self.temporary()?),
            None => None,
        };
        self.emit(Instruction::StartArguments, arguments[0].position());
        for argument in arguments {
            let register = self.temporary()?;
            let (value, add): (&Expression, fn(u8) -> Instruction) = match argument {
                Expression::Splat { expression, .. } => {
                    (expression, |from| Instruction::SplatArgument { from })
                }
                Expression::KeywordSplat { expression, .. } => (expression, |from| {
                    Instruction::KeywordSplatArgument { from }
                }),
                Expression::BlockArg { expression, .. } => {
                    (expression, |from| Instruction::BlockArgument { from })
                }
                plain => (plain, |from| Instruction::PushArgument { from }),
            };
            self.expression(value, register)?;
            self.emit(add(register as u8), argument.position());
            self.next_register = register;
        }
        if let (Some(block), Some(register)) = (trailing_block, block_register) {
            self.expression(block, register)?;
        }
        Ok(site)
    }

    /// Compile `*spread` written as a subscript to an Array of the values it
    /// spreads, in a register kept until the subscript is used, answering
    /// the register.
    fn spread_subscripts(
        &mut self,
        spread: &Expression,
        splat_at: Position,
    ) -> Result<usize, &'static str> {
        let subscripts = self.temporary()?;
        let value = self.temporary()?;
        self.emit(Instruction::StartArguments, splat_at);
        self.expression(spread, value)?;
        self.emit(Instruction::SplatArgument { from: value as u8 }, splat_at);
        self.emit(
            Instruction::ArgumentsToArray {
                to: subscripts as u8,
            },
            splat_at,
        );
        self.next_register = value;
        Ok(subscripts)
    }

    /// Load an Integer, in the instruction when it fits 16 bits.
    fn integer(&mut self, value: i64, to: u8) -> Result<Instruction, &'static str> {
        Ok(match i16::try_from(value) {
            Ok(value) => Instruction::LoadInteger { to, value },
            Err(_) => Instruction::LoadConstant {
                to,
                constant: self.constant(Object::Int(value))?,
            },
        })
    }

    /// Compile a Hash literal's keys and values, in the order written, to
    /// consecutive registers.
    fn hash_shape(
        &mut self,
        entries: &[(&Expression, &Expression)],
    ) -> Result<HashShape, &'static str> {
        let first = self.next_register;
        let mut shape = HashShape {
            first,
            entries: Vec::new(),
        };
        for &(key, value) in entries {
            if let Expression::KeywordSplat {
                expression,
                position,
            } = key
            {
                let register = self.temporary()?;
                self.expression(expression, register)?;
                shape.entries.push(HashEntry::Spread {
                    position: *position,
                });
                continue;
            }
            let key_register = self.temporary()?;
            self.expression(key, key_register)?;
            let value_register = self.temporary()?;
            self.expression(value, value_register)?;
            shape.entries.push(HashEntry::Pair {
                key_position: key.position(),
                value_line: value.position().line,
                literal_key: crate::vm::expression::is_literal_key(key),
            });
        }
        self.next_register = first;
        Ok(shape)
    }

    /// Compile a chunk of an Array literal's elements to consecutive
    /// registers, answering the first.
    fn chunk_of_elements(&mut self, elements: &[Expression]) -> Result<u8, &'static str> {
        let held = self.next_register;
        for element in elements {
            let register = self.temporary()?;
            self.expression(element, register)?;
        }
        self.next_register = held;
        u8::try_from(held).map_err(|_| OVER_LIMIT)
    }

    /// Compile an interpolated String's holes, each to a register followed
    /// by one holding its text.
    fn pieces(&mut self, parts: &[InterpolationPart]) -> Result<Vec<Piece>, &'static str> {
        let mut pieces = Vec::with_capacity(parts.len());
        for part in parts {
            match part {
                InterpolationPart::Text(text) => pieces.push(Piece::Text(text.clone())),
                InterpolationPart::Expression(hole) => {
                    let register = self.temporary()?;
                    let text = self.temporary()?;
                    self.expression(hole, register)?;
                    self.emit(
                        Instruction::ToText {
                            to: text as u8,
                            from: register as u8,
                        },
                        hole.position(),
                    );
                    pieces.push(Piece::Value {
                        register,
                        position: hole.position(),
                    });
                }
            }
        }
        Ok(pieces)
    }

    /// A register for an operand of an arithmetic or comparison
    /// instruction, which names registers below 128 only.
    fn operand_temporary(&mut self, expression: &Expression) -> Result<usize, &'static str> {
        let register = self.temporary()?;
        if register >= usize::from(FIRST_CONSTANT_OPERAND) {
            return Err(expression.kind_name());
        }
        Ok(register)
    }

    fn identifier(&mut self, name: &str, to: u8) -> Result<Instruction, &'static str> {
        if let Some(slot) = self
            .assigned
            .iter()
            .position(|held| held == name)
            .and_then(|_| self.unit.locals.iter().position(|held| held.text == name))
        {
            return Ok(Instruction::GetLocal {
                to,
                slot: wide(Some(slot))?,
            });
        }
        let name_index = self.name(name)?;
        Ok(if starts_uppercase(name) {
            Instruction::GetConstant {
                to,
                name: name_index,
            }
        } else {
            Instruction::GetIdentifier {
                to,
                name: name_index,
            }
        })
    }

    fn name(&mut self, text: &str) -> Result<u16, &'static str> {
        let at = self.unit.name(Name::new(text));
        wide(Some(at))
    }

    fn constant(&mut self, value: Object) -> Result<u16, &'static str> {
        let at = self.unit.constant(value);
        wide(Some(at))
    }
}

/// A 16-bit operand for a side table index.
fn wide(index: Option<usize>) -> Result<u16, &'static str> {
    index
        .and_then(|index| u16::try_from(index).ok())
        .ok_or(OVER_LIMIT)
}

/// Whether an argument is `*value`, `**value`, or `&value`, which a call
/// gathers in an argument list.
fn gathered_in_a_list(argument: &Expression) -> bool {
    matches!(
        argument,
        Expression::Splat { .. } | Expression::KeywordSplat { .. } | Expression::BlockArg { .. }
    )
}

/// Whether a method body's last statement answers the method's value
/// itself, as the tree walker's `terminal_statement_value` does.
fn answers_its_own_value(statement: &Statement) -> bool {
    matches!(
        statement,
        Statement::Expression { .. }
            | Statement::Assignment { .. }
            | Statement::If { .. }
            | Statement::Unless { .. }
    )
}

fn starts_uppercase(name: &str) -> bool {
    name.starts_with(|first: char| first.is_ascii_uppercase())
}

/// How a call's receiver was written.
fn receiver_written(receiver: &Expression) -> Receiver {
    match receiver {
        Expression::SelfExpr { .. } => Receiver::SelfKeyword,
        Expression::Identifier { name, .. } if name == "self" => Receiver::SelfName,
        _ => Receiver::Other,
    }
}

/// The `op` operand `BinaryOperator` applies `op` with.
fn operator_index(op: &BinaryOp) -> u8 {
    BINARY_OPERATORS
        .iter()
        .position(|held| held == op)
        .expect("an operator BinaryOperator applies") as u8
}

type BinaryInstruction = fn(u8, u8, u8) -> Instruction;

/// The instruction applying `op`, for the operators that have one.
fn binary_instruction(op: &BinaryOp) -> Option<BinaryInstruction> {
    Some(match op {
        BinaryOp::Add => |to, left, right| Instruction::Add { to, left, right },
        BinaryOp::Subtract => |to, left, right| Instruction::Subtract { to, left, right },
        BinaryOp::Multiply => |to, left, right| Instruction::Multiply { to, left, right },
        BinaryOp::Divide => |to, left, right| Instruction::Divide { to, left, right },
        BinaryOp::Modulo => |to, left, right| Instruction::Modulo { to, left, right },
        BinaryOp::Equal => |to, left, right| Instruction::Equal { to, left, right },
        BinaryOp::NotEqual => |to, left, right| Instruction::NotEqual { to, left, right },
        BinaryOp::Less => |to, left, right| Instruction::Less { to, left, right },
        BinaryOp::LessEqual => |to, left, right| Instruction::LessEqual { to, left, right },
        BinaryOp::Greater => |to, left, right| Instruction::Greater { to, left, right },
        BinaryOp::GreaterEqual => |to, left, right| Instruction::GreaterEqual { to, left, right },
        _ => return None,
    })
}
