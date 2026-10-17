//! The work an instruction does itself on Integers, Floats, Arrays, and
//! Hashes, while the methods it stands for are the core library's own. A
//! method the program writes in their place, or a refinement in force, sends
//! the instruction back to an ordinary call.

use crate::bytecode::instruction::Instruction;
use crate::class::MethodGuard;
use crate::object::{Object, View};
use crate::vm::VirtualMachine;
use crate::vm::method_execution::written_by_the_program;

/// The operators the arithmetic and comparison instructions apply.
const OPERATOR_NAMES: [&str; 11] = ["+", "-", "*", "/", "%", "==", "!=", "<", "<=", ">", ">="];

thread_local! {
    static INTEGER_OPERATORS: MethodGuard = const { MethodGuard::new() };
    static FLOAT_OPERATORS: MethodGuard = const { MethodGuard::new() };
    static ARRAY_READ: MethodGuard = const { MethodGuard::new() };
    static HASH_READ: MethodGuard = const { MethodGuard::new() };
    static HASH_WRITE: MethodGuard = const { MethodGuard::new() };
}

impl VirtualMachine {
    /// `left` and `right` under the operator `instruction` applies, when both
    /// are Integers or both are Floats and the operator is the core
    /// library's. None leaves the operation to the method call.
    pub(crate) fn specialized_operation(
        &self,
        instruction: &Instruction,
        left: &Object,
        right: &Object,
    ) -> Option<Object> {
        match (left.view(), right.view()) {
            (View::Int(left_value), View::Int(right_value)) => {
                if !self.operators_in_force(&INTEGER_OPERATORS, left) {
                    return None;
                }
                integer_operation(instruction, left_value, right_value)
            }
            (View::Float(left_value), View::Float(right_value)) => {
                if !self.operators_in_force(&FLOAT_OPERATORS, left) {
                    return None;
                }
                float_operation(instruction, left_value, right_value)
            }
            _ => None,
        }
    }

    /// Whether the operators of `sample`'s class are the core library's.
    fn operators_in_force(
        &self,
        guard: &'static std::thread::LocalKey<MethodGuard>,
        sample: &Object,
    ) -> bool {
        !self.any_refinement_active()
            && guard.with(|guard| {
                guard.holds(|| {
                    OPERATOR_NAMES
                        .iter()
                        .all(|name| self.lookup_own_operator_method(sample, name).is_none())
                })
            })
    }

    /// Whether the Integer operators the arithmetic and comparison
    /// instructions apply are the core library's.
    pub(crate) fn integer_operators_in_force(&self) -> bool {
        self.operators_in_force(&INTEGER_OPERATORS, &Object::Int(0))
    }

    /// Whether `[]` on `collection`, an Array, still reads it directly.
    pub(crate) fn array_read_in_force(&self, collection: &Object) -> bool {
        self.core_method_in_force(&ARRAY_READ, collection, "[]")
    }

    /// Whether `[]` on `collection`, a Hash, still reads it directly.
    pub(crate) fn hash_read_in_force(&self, collection: &Object) -> bool {
        self.core_method_in_force(&HASH_READ, collection, "[]")
    }

    /// Whether `[]=` on `collection`, a Hash, still writes it directly.
    pub(crate) fn hash_write_in_force(&self, collection: &Object) -> bool {
        self.core_method_in_force(&HASH_WRITE, collection, "[]=")
    }

    /// Whether `name` on `receiver` reaches the core library's method: no
    /// refinement is in force, and the program wrote no method of the name
    /// on the receiver's singleton class, its class, or a module prepended
    /// to its class.
    fn core_method_in_force(
        &self,
        guard: &'static std::thread::LocalKey<MethodGuard>,
        receiver: &Object,
        name: &str,
    ) -> bool {
        !self.any_refinement_active()
            && guard.with(|guard| {
                guard.holds(|| {
                    let class = self.builtins().class_of(receiver);
                    let mut owners = class.transitive_prepends();
                    owners.push(class);
                    !owners.iter().any(|owner| {
                        owner
                            .find_own_method(name)
                            .is_some_and(|method| written_by_the_program(&method))
                    })
                })
            })
            && self
                .existing_singleton_class(receiver)
                .is_none_or(|singleton| singleton.find_own_method(name).is_none())
    }
}

impl VirtualMachine {
    /// Whether `send`, `__send__`, or `public_send`, named `name`, is the
    /// core library's on `receiver`.
    pub(crate) fn send_in_force(&self, receiver: &Object, name: &str) -> bool {
        !self.any_refinement_active()
            && self
                .lookup_method(receiver, name)
                .is_none_or(|(_, method)| {
                    method.body.is_empty()
                        && method.c_function.is_none()
                        && method.native_alias.is_none()
                        && !method.is_undefined
                })
    }

    /// `receiver.send(:name, ...)` made without looking `send` up: the
    /// arguments start with the name, and `block` is the one written on the
    /// call.
    pub(crate) fn send_literal_name(
        &mut self,
        receiver: Object,
        send: &str,
        arguments: Vec<Object>,
        block: Option<Object>,
        position: crate::lexer::Position,
    ) -> Result<Object, crate::error::MetorexError> {
        let has_block = block.is_some();
        let calling_frame = self.current_method_frame;
        self.refuse_moved_receiver(&receiver, position)?;
        self.attach_block(block);
        let sent = self
            .call_object_sending_method(&receiver, send, &arguments, position)
            .map(|answered| answered.unwrap_or(Object::Nil));
        match sent {
            Err(crate::error::MetorexError::BlockBreak {
                value, home_frame, ..
            }) if has_block && home_frame == calling_frame => Ok(value),
            other => other,
        }
    }
}

fn integer_operation(instruction: &Instruction, left: i64, right: i64) -> Option<Object> {
    Some(match instruction {
        Instruction::Add { .. } => Object::Int(left.checked_add(right)?),
        Instruction::Subtract { .. } => Object::Int(left.checked_sub(right)?),
        Instruction::Multiply { .. } => Object::Int(left.checked_mul(right)?),
        Instruction::Divide { .. } => {
            let quotient = left.checked_div(right)?;
            // Ruby rounds a quotient toward negative infinity.
            let rounded_down = left % right != 0 && (left < 0) != (right < 0);
            Object::Int(if rounded_down { quotient - 1 } else { quotient })
        }
        Instruction::Modulo { .. } => {
            let remainder = left.checked_rem(right)?;
            // Ruby's remainder takes the sign of the divisor.
            let other_sign = remainder != 0 && (remainder < 0) != (right < 0);
            Object::Int(if other_sign {
                remainder + right
            } else {
                remainder
            })
        }
        Instruction::Equal { .. } => Object::Bool(left == right),
        Instruction::NotEqual { .. } => Object::Bool(left != right),
        Instruction::Less { .. } => Object::Bool(left < right),
        Instruction::LessEqual { .. } => Object::Bool(left <= right),
        Instruction::Greater { .. } => Object::Bool(left > right),
        Instruction::GreaterEqual { .. } => Object::Bool(left >= right),
        _ => return None,
    })
}

fn float_operation(instruction: &Instruction, left: f64, right: f64) -> Option<Object> {
    Some(match instruction {
        Instruction::Add { .. } => Object::Float(left + right),
        Instruction::Subtract { .. } => Object::Float(left - right),
        Instruction::Multiply { .. } => Object::Float(left * right),
        Instruction::Divide { .. } => Object::Float(left / right),
        Instruction::Equal { .. } => Object::Bool(left == right),
        Instruction::NotEqual { .. } => Object::Bool(left != right),
        Instruction::Less { .. } => Object::Bool(left < right),
        Instruction::LessEqual { .. } => Object::Bool(left <= right),
        Instruction::Greater { .. } => Object::Bool(left > right),
        Instruction::GreaterEqual { .. } => Object::Bool(left >= right),
        _ => return None,
    })
}
