// The smallest and largest a range holds, and whether it covers a value.

use super::*;
use crate::object::{OptionView, View};

impl VirtualMachine {
    /// The smallest and largest a range holds, and whether it covers a value.
    pub(crate) fn call_range_bounds_method(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        let View::Range {
            start,
            end,
            exclusive,
            ..
        } = receiver.view()
        else {
            return Ok(None);
        };
        match method_name {
            // `min` and `max` order the values, and a block decides the order
            // the way it does for an array.
            "min" | "max" | "minmax" => {
                if matches!(start.view(), View::Nil) && method_name != "max" {
                    return Err(crate::vm::errors::simple_exception(
                        "RangeError",
                        "cannot get the minimum of beginless range",
                        position,
                    ));
                }
                if matches!(end.view(), View::Nil) && method_name != "min" {
                    return Err(crate::vm::errors::simple_exception(
                        "RangeError",
                        "cannot get the maximum of endless range",
                        position,
                    ));
                }
                let has_block = matches!(self.pending_block.view(), Some(View::Block(_)));
                // A beginless range has no value to start a comparison from,
                // so a custom order cannot pick a largest value.
                if matches!(start.view(), View::Nil) && has_block {
                    return Err(crate::vm::errors::simple_exception(
                        "RangeError",
                        "cannot get the maximum of beginless range with custom comparison method",
                        position,
                    ));
                }
                // Both ends at once are answered by asking for each on its
                // own, which Range does in the core library.
                if method_name == "minmax" && arguments.is_empty() {
                    return Ok(None);
                }
                if let Some(count) = arguments
                    .first()
                    .filter(|held| !matches!(held.view(), View::Nil))
                    && !has_block
                    && method_name != "minmax"
                    && let Some(counted) = self.counted_range_bounds(
                        method_name,
                        start,
                        end,
                        exclusive,
                        count,
                        position,
                    )?
                {
                    return Ok(Some(counted));
                }
                // Without a block or a count, the ends answer directly, which
                // works for a range too large to walk.
                if !has_block && arguments.is_empty() && method_name != "minmax" {
                    if method_name == "min" {
                        let order = self.evaluate_binary_operation(
                            &crate::ast::BinaryOp::Spaceship,
                            (*start).clone(),
                            (*end).clone(),
                            position,
                        )?;
                        if matches!(order.view(), View::Int(value) if value > 0) {
                            return Ok(Some(Object::Nil));
                        }
                        // A range that leaves its end out and begins there
                        // holds nothing at all.
                        if exclusive && matches!(order.view(), View::Int(0)) {
                            return Ok(Some(Object::Nil));
                        }
                        return Ok(Some((*start).clone()));
                    }
                    // Leaving the end out only makes sense over whole
                    // numbers, where the value below it is known.
                    let last = end.as_big_integer();
                    let ends_beginless = matches!(start.view(), View::Nil);
                    if exclusive && last.is_none() && (ends_beginless || counts_as_a_number(end)) {
                        return Err(crate::vm::errors::simple_exception(
                            "TypeError",
                            "cannot exclude non Integer end value",
                            position,
                        ));
                    }
                    let begins_whole = start.as_big_integer().is_some();
                    if ends_beginless {
                        if let (true, Some(reached)) = (exclusive, last.as_ref()) {
                            return Ok(Some(Object::integer(reached.clone() - 1)));
                        }
                        return Ok(Some((*end).clone()));
                    }
                    let order = self.evaluate_binary_operation(
                        &crate::ast::BinaryOp::Spaceship,
                        (*start).clone(),
                        (*end).clone(),
                        position,
                    )?;
                    if matches!(order.view(), View::Int(value) if value > 0) {
                        return Ok(Some(Object::Nil));
                    }
                    if !exclusive {
                        return Ok(Some((*end).clone()));
                    }
                    if let Some(reached) = last {
                        if !begins_whole {
                            return Err(crate::vm::errors::simple_exception(
                                "TypeError",
                                "cannot exclude end value with non Integer begin value",
                                position,
                            ));
                        }
                        if matches!(order.view(), View::Int(0)) {
                            return Ok(Some(Object::Nil));
                        }
                        return Ok(Some(Object::integer(reached - 1)));
                    }
                }
                let elements = self.range_elements(start, end, exclusive, position)?;
                let walked = Object::array(elements);
                self.send_to_object(walked, method_name, arguments.to_vec(), position)
                    .map(Some)
            }
            // `cover?` asks whether a value falls between the ends, which
            // needs no walk, and takes another Range too. `===` picks the
            // same answer, which is how a Range reads in a `case`.
            "cover?" | "===" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                if method_name == "cover?"
                    && let View::Range {
                        start: other_start,
                        end: other_end,
                        exclusive: other_exclusive,
                        ..
                    } = arguments[0].view()
                {
                    let covered = self.range_covers_range(
                        (start, end, exclusive),
                        &arguments[0],
                        (other_start, other_end, &other_exclusive),
                        position,
                    )?;
                    return Ok(Some(Object::Bool(covered)));
                }
                let covered = self.range_covers(start, end, exclusive, &arguments[0], position)?;
                Ok(Some(Object::Bool(covered)))
            }
            // `overlap?` asks whether the two ranges share any value.
            "overlap?" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let View::Range {
                    start: other_start,
                    end: other_end,
                    exclusive: other_exclusive,
                    ..
                } = arguments[0].view()
                else {
                    let message = format!(
                        "wrong argument type {} (expected Range)",
                        self.builtins().class_of(&arguments[0]).ruby_name()
                    );
                    return Err(crate::vm::errors::simple_exception(
                        "TypeError",
                        &message,
                        position,
                    ));
                };
                // A range holding no value shares none, and two ranges over
                // values that do not compare share none either.
                if self.range_holds_nothing(start, end, exclusive, position)?
                    || self.range_holds_nothing(
                        other_start,
                        other_end,
                        other_exclusive,
                        position,
                    )?
                {
                    return Ok(Some(Object::Bool(false)));
                }
                if !matches!(start.view(), View::Nil) && !matches!(other_start.view(), View::Nil) {
                    let compared = self.evaluate_binary_operation(
                        &crate::ast::BinaryOp::Spaceship,
                        (*start).clone(),
                        (*other_start).clone(),
                        position,
                    )?;
                    if matches!(compared.view(), View::Nil) {
                        return Ok(Some(Object::Bool(false)));
                    }
                }
                let before = self.range_ends_before(end, exclusive, other_start, position)?;
                let after = self.range_ends_before(other_end, other_exclusive, start, position)?;
                Ok(Some(Object::Bool(!before && !after)))
            }
            _ => Ok(None),
        }
    }

    /// `min(n)` or `max(n)` over a range of Integers, counted from the end
    /// asked for without walking the range, which may be too large to walk.
    /// None leaves any other range to be walked.
    fn counted_range_bounds(
        &mut self,
        method_name: &str,
        start: &Object,
        end: &Object,
        exclusive: bool,
        count: &Object,
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        let count = self.coerce_integer_argument(count, position)?;
        if count < num_bigint::BigInt::from(0) {
            return Err(crate::vm::errors::simple_exception(
                "ArgumentError",
                "negative array size (or size too big)",
                position,
            ));
        }
        let count = usize::try_from(count).unwrap_or(usize::MAX);
        let first = start.as_big_integer();
        let last = end.as_big_integer();
        let open_end = if method_name == "max" { start } else { end };
        let counted_from = if method_name == "max" { end } else { start };
        if matches!(open_end.view(), View::Nil) && counted_from.as_big_integer().is_none() {
            let class_name = match start.view() {
                View::Nil => "NilClass".to_string(),
                _ => self.builtins().class_of(start).ruby_name(),
            };
            let message = format!("can't iterate from {class_name}");
            return Err(crate::vm::errors::simple_exception(
                "TypeError",
                &message,
                position,
            ));
        }
        let last = last.map(|value| if exclusive { value - 1 } else { value });
        let mut values = Vec::new();
        match (method_name, first, last) {
            ("max", first, Some(mut at))
                if first.is_some() || matches!(start.view(), View::Nil) =>
            {
                while values.len() < count && first.as_ref().is_none_or(|first| at >= *first) {
                    values.push(Object::integer(at.clone()));
                    at -= 1;
                }
            }
            ("min", Some(mut at), last) if last.is_some() || matches!(end.view(), View::Nil) => {
                while values.len() < count && last.as_ref().is_none_or(|last| at <= *last) {
                    values.push(Object::integer(at.clone()));
                    at += 1;
                }
            }
            _ => return Ok(None),
        }
        Ok(Some(Object::array(values)))
    }
    /// Whether every value of the range `other` falls within this one, as
    /// Ruby's `r_cover_range_p` decides it. A range holding nothing, such as
    /// a backward one, is covered by none.
    fn range_covers_range(
        &mut self,
        (start, end, exclusive): (&Object, &Object, bool),
        other: &Object,
        (other_start, other_end, other_exclusive): (&Object, &Object, &bool),
        position: Position,
    ) -> Result<bool, MetorexError> {
        let open = |held: &Object| matches!(held.view(), View::Nil);
        if (!open(end) && open(other_end)) || (!open(start) && open(other_start)) {
            return Ok(false);
        }
        if !open(other_start) && !open(other_end) {
            let order = self.order_of(other_start, other_end, position)?;
            if order > if *other_exclusive { -1 } else { 0 } {
                return Ok(false);
            }
        }
        if !open(other_start) && !self.range_covers(start, end, exclusive, other_start, position)? {
            return Ok(false);
        }
        let end_order = self.order_of(end, other_end, position)?;
        if exclusive == *other_exclusive {
            return Ok(end_order >= 0);
        }
        if exclusive {
            return Ok(end_order > 0);
        }
        if end_order >= 0 {
            return Ok(true);
        }
        // An inclusive range covers an exclusive one ending past it when the
        // largest value the other holds is still within it.
        // A range whose largest value cannot be named, as one ending on a
        // Float it leaves out, is not covered.
        let largest = match self.send_to_object(other.clone(), "max", Vec::new(), position) {
            Err(MetorexError::UncaughtException { exception, .. })
                if self.exception_matches(&exception, &["TypeError".to_string()])? =>
            {
                Object::Nil
            }
            answered => answered?,
        };
        if open(&largest) {
            return Ok(false);
        }
        Ok(self.order_of(end, &largest, position)? >= 0)
    }

    /// Where `left` stands against `right` by `<=>`, with values that do
    /// not compare counted as greater, as Ruby's `r_less` counts them.
    fn order_of(
        &mut self,
        left: &Object,
        right: &Object,
        position: Position,
    ) -> Result<i64, MetorexError> {
        let order = self.evaluate_binary_operation(
            &crate::ast::BinaryOp::Spaceship,
            left.clone(),
            right.clone(),
            position,
        )?;
        Ok(match order.view() {
            View::Int(value) => value.signum(),
            _ => i64::MAX,
        })
    }
}
