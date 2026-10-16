//! Compiling a body's statements to a `Unit`. A body holding a node the
//! compiler does not handle yet is refused with that node's type name, and
//! runs on the tree walker.

use crate::ast::{BinaryOp, Expression, InterpolationPart, Statement, UnaryOp};
use crate::bytecode::instruction::{
    FIRST_CONSTANT_OPERAND, Instruction, RANGE_EXCLUSIVE, RANGE_LITERAL_ENDS,
};
use crate::bytecode::unit::{
    CallSite, ErrorSpot, HashEntry, HashShape, Name, Piece, Receiver, RegexLiteral, Unit,
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
            } => self.assignment(target, value, *position),
            other => Err(other.kind_name()),
        }
    }

    fn assignment(
        &mut self,
        target: &Expression,
        value: &Expression,
        position: Position,
    ) -> Result<(), &'static str> {
        let from = ANSWER as u8;
        let instruction = match target {
            Expression::Identifier { name, .. } if self.assigned.contains(name) => {
                Instruction::SetLocal {
                    from,
                    slot: wide(self.unit.locals.iter().position(|held| &held.text == name))?,
                }
            }
            Expression::Identifier { name, .. } if starts_uppercase(name) => {
                Instruction::SetConstant {
                    from,
                    name: self.name(name)?,
                }
            }
            Expression::InstanceVariable { name, .. } => Instruction::SetInstanceVariable {
                from,
                name: self.name(name)?,
            },
            Expression::GlobalVariable { name, .. } => Instruction::SetGlobal {
                from,
                name: self.name(name)?,
            },
            Expression::ClassVariable { name, .. } => Instruction::SetClassVariable {
                from,
                name: self.name(name)?,
            },
            // An assignment to any other target is refused as a whole.
            _ => return Err("Assignment"),
        };
        self.expression(value, ANSWER)?;
        self.emit(instruction, position);
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
                op, left, right, ..
            } => {
                let Some(make) = binary_instruction(op) else {
                    return Err(expression.kind_name());
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
                    UnaryOp::Plus => return Err(expression.kind_name()),
                }
            }
            Expression::Array { elements, .. } => {
                if let Some(splat) = elements
                    .iter()
                    .find(|element| matches!(element, Expression::Splat { .. }))
                {
                    return Err(splat.kind_name());
                }
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
                trailing_block: None,
                ..
            } if method != "[]" && method != crate::parser::SAFE_CALL => {
                let base = self.temporary()?;
                self.expression(receiver, base)?;
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
                let site = CallSite::new(method, arguments.len(), receiver_written(receiver));
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
                return Ok(());
            }
            other => return Err(other.kind_name()),
        };
        self.emit(instruction, position);
        Ok(())
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
