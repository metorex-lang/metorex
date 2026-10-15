// The `+` and `-` written in front of a value.

use super::*;
use crate::object::View;

impl VirtualMachine {
    /// The method a program of its own put on Integer or Float for an
    /// operator, or None where the numbers answer for themselves.
    pub(crate) fn redefined_number_operator(
        &mut self,
        held: &Object,
        named: &str,
    ) -> Option<(Rc<crate::class::Class>, Rc<crate::object::Method>)> {
        let class = self.builtins().class_of(held);
        let found = class.find_own_method(named)?;
        if found.is_undefined || found.body.is_empty() {
            return None;
        }
        Some((class, found))
    }

    /// The arithmetic a number answers for an operator on its own, reached
    /// without consulting a redefinition. An UnboundMethod cut from Integer
    /// before the program replaced `+` calls through here, so binding it back
    /// runs the original rather than the replacement.
    pub(crate) fn builtin_number_operator(
        &mut self,
        named: &str,
        left: Object,
        right: Object,
        position: Position,
    ) -> Option<Result<Object, MetorexError>> {
        if !matches!(left.view(), View::Int(_) | View::BigInt(_) | View::Float(_)) {
            return None;
        }
        // Anything but a number on the right is asked to `coerce`, which is
        // the ordinary dispatch rather than the arithmetic underneath.
        if !matches!(
            right.view(),
            View::Int(_) | View::BigInt(_) | View::Float(_)
        ) {
            return None;
        }
        match named {
            "+" => Some(self.evaluate_addition(left, right, position)),
            "-" => Some(self.evaluate_numeric_binary(&BinaryOp::Subtract, left, right, position)),
            "*" => Some(self.evaluate_numeric_binary(&BinaryOp::Multiply, left, right, position)),
            "/" => Some(self.evaluate_numeric_binary(&BinaryOp::Divide, left, right, position)),
            "%" => Some(self.evaluate_numeric_binary(&BinaryOp::Modulo, left, right, position)),
            "**" => Some(self.evaluate_numeric_binary(&BinaryOp::Power, left, right, position)),
            _ => None,
        }
    }

    /// Evaluate a unary operation (`+` or `-`).
    pub(crate) fn evaluate_unary_operation(
        &mut self,
        op: &UnaryOp,
        value: Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        match op {
            UnaryOp::Plus => match value.view() {
                // Numeric `+x` is a no-op identity.
                View::Int(_) | View::BigInt(_) | View::Float(_) => Ok(value),
                // `+str` asks for a string that changes, so a frozen one and
                // one carrying notice that it will be frozen both answer a
                // copy, and any other answers itself.
                View::String(text) => {
                    if !text.is_frozen() && !text.is_chilled() {
                        return Ok(value.clone());
                    }
                    Ok(Object::String(std::rc::Rc::new(
                        crate::object::StringValue::with_encoding(
                            text.to_text(),
                            text.encoding_name(),
                        ),
                    )))
                }
                _ => Err(unary_type_error(op, &value, position)),
            },
            UnaryOp::Minus => match value.view() {
                // Negating i64::MIN needs the wider type.
                View::Int(v) => Ok(Object::integer(-num_bigint::BigInt::from(v))),
                View::BigInt(v) => Ok(Object::integer(-num_bigint::BigInt::clone(v))),
                View::Float(v) => Ok(Object::Float(-v)),
                // `-str` is `String#-@`, which hands back the one frozen copy
                // every equal string shares.
                View::String(_) => self.send_to_object(value, "-@", vec![], position),
                _ => Err(unary_type_error(op, &value, position)),
            },
            UnaryOp::Not => Ok(Object::Bool(matches!(
                value.view(),
                View::Bool(false) | View::Nil
            ))),
        }
    }
}
