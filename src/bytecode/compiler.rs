//! Compiling a body's statements to a `Unit`. A body holding a node the
//! compiler does not handle yet is refused with that node's type name, and
//! runs on the tree walker.

use crate::ast::{BinaryOp, Expression, InterpolationPart, Statement, UnaryOp};
use crate::bytecode::instruction::{
    BINARY_OPERATORS, FIRST_CONSTANT_OPERAND, Instruction, RANGE_EXCLUSIVE, RANGE_LITERAL_ENDS,
};
use crate::bytecode::unit::{
    CallSite, ErrorSpot, HashEntry, HashShape, MultipleAssignment, Name, Piece, Receiver,
    RegexLiteral, Target, Unit,
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
    let mut compiler = Compiler {
        unit: Unit::new(),
        next_register: ANSWER + 1,
        spots: Vec::new(),
        assigned: crate::ast::collect_assigned_locals(body),
    };
    compiler.unit.use_registers(ANSWER + 1);
    for name in compiler.assigned.clone() {
        compiler.unit.local(&name);
    }
    let end = match body.last() {
        Some(last) => last.position(),
        None => Position::default(),
    };
    if body.is_empty() {
        compiler.emit(Instruction::LoadNil { to: ANSWER as u8 }, end);
    }
    for statement in body {
        let start = compiler.unit.instructions.len();
        compiler
            .statement(statement)
            .map_err(naming(statement.kind_name()))?;
        compiler.unit.start_statement(start, statement.position());
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
}

impl Compiler {
    /// Add an instruction, with the spot of the innermost expression being
    /// compiled that has one.
    fn emit(&mut self, instruction: Instruction, position: Position) -> usize {
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

    fn statement(&mut self, statement: &Statement) -> Result<(), &'static str> {
        match statement {
            Statement::Expression { expression, .. } => self.expression(expression, ANSWER),
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
                let compiled = self.assignment(target, value, ANSWER, *position);
                self.spots.pop();
                compiled
            }
            Statement::MultipleAssignment {
                targets,
                values,
                position,
            } => self.multiple_assignment(targets, values, *position),
            Statement::DeclareLocals { names, position } => {
                for name in names {
                    let slot = wide(Some(self.unit.local(name)))?;
                    self.emit(Instruction::DeclareLocal { slot }, *position);
                }
                self.emit(Instruction::LoadNil { to: ANSWER as u8 }, *position);
                Ok(())
            }
            other => Err(other.kind_name()),
        }
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
        position: Position,
    ) -> Result<(), &'static str> {
        let held = self.next_register;
        let mut prepared = Vec::with_capacity(targets.len());
        for target in targets {
            prepared.push(self.prepare(target)?);
        }
        let lone_value = match values {
            [single] if !matches!(single, Expression::Splat { .. }) => {
                self.expression(single, ANSWER)?;
                true
            }
            _ => {
                let gathered = Expression::Array {
                    elements: values.to_vec(),
                    position,
                };
                self.expression(&gathered, ANSWER)?;
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
                from: ANSWER as u8,
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
        let here = wide(Some(self.unit.instructions.len()))?;
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
