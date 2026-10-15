// The operators that read a value bit by bit or as a set.

use super::*;
use crate::object::View;

impl VirtualMachine {
    /// The `&` operator, which is a conjunction on numbers and an
    /// intersection on collections.
    pub(crate) fn evaluate_bitwise_and(
        &mut self,
        left: Object,
        right: Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        use BinaryOp::*;
        match (left.view(), right.view()) {
            // Array intersection, keeping the left operand's order and
            // dropping duplicates.
            (View::Array(left_items), View::Array(right_items)) => {
                // Ruby matches on `hash` and `eql?` here, so an object of
                // the program's own decides for itself.
                let right_items = right_items.borrow().clone();
                let left_items = left_items.borrow().clone();
                let mut intersection: Vec<Object> = Vec::new();
                for item in &left_items {
                    if self.contains_eql(&right_items, item, position)?
                        && !self.contains_eql(&intersection, item, position)?
                    {
                        intersection.push(item.clone());
                    }
                }
                Ok(Object::array(intersection))
            }
            // nil & x always returns false (Ruby semantics)
            (View::Nil, _) => Ok(Object::Bool(false)),
            (View::Bool(a), View::Bool(b)) => Ok(Object::Bool(a & b)),
            (
                ref left @ (View::Int(_) | View::BigInt(_)),
                ref right @ (View::Int(_) | View::BigInt(_)),
            ) => Ok(Object::integer(
                left.as_big_integer().expect("integer-kinded")
                    & right.as_big_integer().expect("integer-kinded"),
            )),
            (View::Bool(a), other) => Ok(Object::Bool(a & other.is_truthy())),
            (lhs, rhs) => Err(binary_type_error(
                BitwiseAnd,
                &lhs.to_object(),
                &rhs.to_object(),
                position,
            )),
        }
    }

    /// The `|` operator, which is a disjunction on numbers and a union on
    /// collections.
    pub(crate) fn evaluate_bitwise_or(
        &mut self,
        left: Object,
        right: Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        use BinaryOp::*;
        match (left.view(), right.view()) {
            // Array union, keeping first-seen order and dropping
            // duplicates.
            (View::Array(left_items), View::Array(right_items)) => {
                let mut all = left_items.borrow().clone();
                all.extend(right_items.borrow().iter().cloned());
                let mut union: Vec<Object> = Vec::new();
                for item in &all {
                    if !self.contains_eql(&union, item, position)? {
                        union.push(item.clone());
                    }
                }
                Ok(Object::array(union))
            }
            // nil | x returns truthiness of x
            (View::Nil, other) => Ok(Object::Bool(other.is_truthy())),
            (View::Bool(a), View::Bool(b)) => Ok(Object::Bool(a | b)),
            (
                ref left @ (View::Int(_) | View::BigInt(_)),
                ref right @ (View::Int(_) | View::BigInt(_)),
            ) => Ok(Object::integer(
                left.as_big_integer().expect("integer-kinded")
                    | right.as_big_integer().expect("integer-kinded"),
            )),
            (View::Bool(a), other) => Ok(Object::Bool(a | other.is_truthy())),
            (lhs, rhs) => Err(binary_type_error(
                BitwiseOr,
                &lhs.to_object(),
                &rhs.to_object(),
                position,
            )),
        }
    }

    /// The `^` operator.
    pub(crate) fn evaluate_exclusive_or(
        &mut self,
        left: Object,
        right: Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        use BinaryOp::*;
        match (left.view(), right.view()) {
            // nil ^ x returns truthiness of x
            (View::Nil, other) => Ok(Object::Bool(other.is_truthy())),
            (View::Bool(a), View::Bool(b)) => Ok(Object::Bool(a ^ b)),
            (
                ref left @ (View::Int(_) | View::BigInt(_)),
                ref right @ (View::Int(_) | View::BigInt(_)),
            ) => Ok(Object::integer(
                left.as_big_integer().expect("integer-kinded")
                    ^ right.as_big_integer().expect("integer-kinded"),
            )),
            (View::Bool(a), other) => Ok(Object::Bool(a ^ other.is_truthy())),
            (lhs, rhs) => Err(binary_type_error(
                Xor,
                &lhs.to_object(),
                &rhs.to_object(),
                position,
            )),
        }
    }
}
