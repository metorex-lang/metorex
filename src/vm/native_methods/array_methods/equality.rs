// Whether two values count as the same element.

use super::*;
use crate::object::View;

impl VirtualMachine {
    /// Whether two values are `eql?`, which is stricter than `==`: 1 and 1.0
    /// are equal but not eql, and an object of its own decides by answering
    /// `eql?` itself.
    /// Whether two arrays hold equal elements. Ruby asks each pair with
    /// `equal?` before `==`, and an array that reaches itself compares as
    /// equal rather than recursing forever.
    pub(crate) fn arrays_equal(
        &mut self,
        left: &Object,
        right: &Object,
        position: Position,
    ) -> Result<bool, MetorexError> {
        let mut in_flight = Vec::new();
        self.arrays_equal_within(left, right, &mut in_flight, position)
    }

    pub(crate) fn values_eql(
        &mut self,
        left: &Object,
        right: &Object,
        position: Position,
    ) -> Result<bool, MetorexError> {
        let mut in_flight = Vec::new();
        self.values_eql_within(left, right, &mut in_flight, position)
    }

    /// `values_eql` with the pairs already being compared, so a pair of arrays
    /// that reach themselves is taken as equal rather than recursing forever.
    pub(crate) fn values_eql_within(
        &mut self,
        left: &Object,
        right: &Object,
        in_flight: &mut Vec<(usize, usize)>,
        position: Position,
    ) -> Result<bool, MetorexError> {
        // An object of the program's own making answers `eql?` itself. Only
        // the receiver decides, so a mock on the right of an Array is never
        // asked, which is what Ruby does.
        if matches!(left.view(), View::Instance(_))
            && crate::vm::native_methods::array_subclass_value(left).is_none()
            && crate::vm::native_methods::string_subclass_value(left).is_none()
        {
            if let Some((class, method)) = self.lookup_method(left, "eql?")
                && !method.is_undefined
                && !method.body.is_empty()
            {
                let answer =
                    self.invoke_method(class, method, left.clone(), vec![right.clone()], position)?;
                return Ok(answer.is_truthy());
            }
            // A Struct or Data value answers `eql?` natively by its members,
            // and Object's own is identity, where a class that defines none
            // lands.
            if identical(left, right) {
                return Ok(true);
            }
            let answer =
                self.send_to_object(left.clone(), "eql?", vec![right.clone()], position)?;
            return Ok(answer.is_truthy());
        }
        // A String subclass holds its text behind the instance, and Ruby
        // compares that text rather than the two objects.
        let left = &crate::vm::native_methods::string_subclass_value(left)
            .unwrap_or_else(|| elements_of(left).unwrap_or_else(|| left.clone()));
        let right = &crate::vm::native_methods::string_subclass_value(right)
            .unwrap_or_else(|| elements_of(right).unwrap_or_else(|| right.clone()));
        if let (View::String(text), View::String(other)) = (left.view(), right.view()) {
            use crate::vm::native_methods::string_methods::{binary_bytes, strings_comparable};
            return Ok((binary_bytes(text) == binary_bytes(other)
                || *text.as_str() == *other.as_str())
                && strings_comparable(text, other));
        }
        match (left.view(), right.view()) {
            (View::Array(_), View::Array(_)) => {}
            (View::Array(_), _) | (_, View::Array(_)) => return Ok(false),
            _ => {}
        }
        if let (View::Array(left_elements), View::Array(right_elements)) =
            (left.view(), right.view())
        {
            let pair = (
                Rc::as_ptr(left_elements) as usize,
                Rc::as_ptr(right_elements) as usize,
            );
            if pair.0 == pair.1 || in_flight.contains(&pair) {
                return Ok(true);
            }
            let left_elements = left_elements.borrow().clone();
            let right_elements = right_elements.borrow().clone();
            if left_elements.len() != right_elements.len() {
                return Ok(false);
            }
            in_flight.push(pair);
            for (one, other) in left_elements.iter().zip(right_elements.iter()) {
                if !self.values_eql_within(one, other, in_flight, position)? {
                    in_flight.pop();
                    return Ok(false);
                }
            }
            in_flight.pop();
            return Ok(true);
        }
        let same_kind = match (left.view(), right) {
            (View::Float(_), other) => matches!(other.view(), View::Float(_)),
            (View::Int(_) | View::BigInt(_), other) => {
                matches!(other.view(), View::Int(_) | View::BigInt(_))
            }
            _ => true,
        };
        Ok(same_kind && left.equals(right))
    }

    /// Whether an element equals what a search was given. Ruby sends `==` to
    /// the element, so an object of the program's own making answers.
    pub(crate) fn elements_equal(
        &mut self,
        element: &Object,
        candidate: &Object,
        position: Position,
    ) -> Result<bool, MetorexError> {
        if let (View::Array(_), View::Array(_)) = (element.view(), candidate.view()) {
            return self.arrays_equal_within(element, candidate, &mut Vec::new(), position);
        }
        if matches!(element.view(), View::Instance(_) | View::Regex(..))
            || matches!(candidate.view(), View::Instance(_) | View::Regex(..))
        {
            let answer = self.evaluate_binary_operation(
                &crate::ast::BinaryOp::Equal,
                element.clone(),
                candidate.clone(),
                position,
            )?;
            return Ok(answer.is_truthy());
        }
        if let (View::String(text), View::String(other)) = (element.view(), candidate.view()) {
            return Ok(crate::vm::native_methods::string_methods::strings_equal(
                text, other,
            ));
        }
        Ok(element.equals(candidate))
    }

    pub(crate) fn arrays_equal_within(
        &mut self,
        left: &Object,
        right: &Object,
        in_flight: &mut Vec<(usize, usize)>,
        position: Position,
    ) -> Result<bool, MetorexError> {
        let (View::Array(one), View::Array(other)) = (left.view(), right.view()) else {
            return Ok(false);
        };
        let pair = (Rc::as_ptr(one) as usize, Rc::as_ptr(other) as usize);
        if pair.0 == pair.1 || in_flight.contains(&pair) {
            return Ok(true);
        }
        let held = one.borrow().clone();
        let against = other.borrow().clone();
        if held.len() != against.len() {
            return Ok(false);
        }
        in_flight.push(pair);
        let mut same = true;
        for (item, counterpart) in held.iter().zip(against.iter()) {
            if identical(item, counterpart) {
                continue;
            }
            let matched = if matches!(
                (item.view(), counterpart.view()),
                (View::Array(_), View::Array(_))
            ) {
                self.arrays_equal_within(item, counterpart, in_flight, position)?
            } else {
                self.evaluate_binary_operation(
                    &crate::ast::BinaryOp::Equal,
                    item.clone(),
                    counterpart.clone(),
                    position,
                )?
                .is_truthy()
            };
            if !matched {
                same = false;
                break;
            }
        }
        in_flight.pop();
        Ok(same)
    }
}

/// Whether two values are the same object, which is what `equal?` reports.
pub(crate) fn identical(left: &Object, right: &Object) -> bool {
    match (left.view(), right.view()) {
        (View::Instance(one), View::Instance(other)) => Rc::ptr_eq(one, other),
        (View::Array(one), View::Array(other)) => Rc::ptr_eq(one, other),
        (View::Dict(one), View::Dict(other)) => Rc::ptr_eq(one, other),
        // A float is its own bits, which is how an array holding NaN equals
        // another holding the same NaN even though NaN equals nothing.
        (View::Float(one), View::Float(other)) => one.to_bits() == other.to_bits(),
        _ => false,
    }
}
