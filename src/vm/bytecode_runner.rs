//! Running a compiled unit: one loop dispatching each instruction by
//! `match`, over a register file the unit's size. An instruction that needs
//! the interpreter, to call a method, read a constant, or raise, calls the
//! same `VirtualMachine` functions the tree walker does.

use crate::ast::{BinaryOp, Expression, UnaryOp};
use crate::bytecode::instruction::{Instruction, Operand};
use crate::bytecode::unit::{Receiver, Unit};
use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use crate::vm::VirtualMachine;
use crate::vm::method_lookup::WrittenReceiver;

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

impl VirtualMachine {
    /// Run `unit`, answering what its `Return` hands back.
    pub fn run_unit(&mut self, unit: &Unit) -> Result<Object, MetorexError> {
        let mut registers = vec![Object::Nil; unit.register_count];
        let mut at = 0;
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
            let ran = if unit.statement_starts[at] {
                self.begin_statement(position, true).and_then(|()| {
                    self.execute_instruction(unit, &mut registers, instruction, extension, position)
                })
            } else {
                self.execute_instruction(unit, &mut registers, instruction, extension, position)
            };
            extension = Extension::default();
            match ran {
                Ok(Next::Step) => at += 1,
                Ok(Next::JumpTo(target)) => at = target,
                Ok(Next::Finish(value)) => return Ok(value),
                Err(error) => {
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
                let written = match site.receiver {
                    Receiver::SelfKeyword => WrittenReceiver::SelfKeyword,
                    Receiver::Other => WrittenReceiver::Other,
                };
                registers[base] =
                    self.call_evaluated_method(receiver, &site.name, arguments, written, position)?;
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
                registers[register(to)] = self.eval_identifier(name, position)?;
            }
            Instruction::SetConstant { from, name } => {
                let name = unit.names[index(name)].text.clone();
                let target = Expression::Identifier { name, position };
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
                registers[register(to)] = Object::array(elements);
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
