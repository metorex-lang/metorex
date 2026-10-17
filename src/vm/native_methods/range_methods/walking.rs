// Walking the values a range covers, and how many there are.

use super::*;
use crate::object::{OptionView, View};

impl VirtualMachine {
    /// Walking the values a range covers, and how many there are.
    pub(crate) fn call_range_walking_method(
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
            // block answers zero for.
            "bsearch" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let bounds = SearchBounds::of(start, end, exclusive, position)?;
                let viewed = self.pending_block.take();
                let Some(View::Block(block)) = viewed.view() else {
                    return self
                        .make_enumerator(receiver, method_name, arguments, position)
                        .map(Some);
                };
                self.binary_search(&bounds, block, position).map(Some)
            }
            "each" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let pending = self.pending_block.take();
                let block = match pending.view() {
                    Some(View::Block(b)) => b,
                    Some(other) => {
                        return Err(method_argument_type_error(
                            method_name,
                            "Block",
                            &other.to_object(),
                            position,
                        ));
                    }
                    // Without a block the walk is handed back, which is what
                    // `(1..3).each` answers. It counts as many values as the
                    // range holds, where the range can say.
                    None => {
                        let counted = match self
                            .send_to_object(receiver.clone(), "size", vec![], position)
                            .as_ref()
                            .map(Object::view)
                        {
                            Ok(View::Int(count)) => Some(count),
                            _ => None,
                        };
                        return self
                            .build_enumerator(
                                receiver.clone(),
                                method_name,
                                Vec::new(),
                                counted,
                                position,
                            )
                            .map(Some);
                    }
                };

                // A range that counts up from an Integer with no end in sight
                // is walked one value at a time, since there is no list of
                // them to collect. The block stops it with `break`.
                if let Some(first) = endless_int_start(start, end) {
                    let first = &first;
                    let mut current = *first;
                    loop {
                        match self.execute_block_with_control_flow(
                            block,
                            vec![Object::Int(current)],
                            position,
                        )? {
                            crate::vm::ControlFlow::Next
                            | crate::vm::ControlFlow::Value(_)
                            | crate::vm::ControlFlow::Redo { .. }
                            | crate::vm::ControlFlow::Retry { .. }
                            | crate::vm::ControlFlow::Continue { .. } => {}
                            crate::vm::ControlFlow::Break { value, .. } => {
                                return Ok(Some(value));
                            }
                            crate::vm::ControlFlow::Return { value, position } => {
                                return Err(MetorexError::NonLocalReturn {
                                    value,
                                    location: crate::vm::utils::position_to_location(position),
                                    home_frame: block.home_frame,
                                });
                            }
                        }
                        current += 1;
                    }
                }

                // A String range with no end follows `succ` for as long as
                // the block keeps asking, the same way the Integer one counts.
                if let Some(first) = endless_string_start(start, end) {
                    let mut current = first;
                    loop {
                        let value = Object::string(current.clone());
                        match self.execute_block_with_control_flow(block, vec![value], position)? {
                            crate::vm::ControlFlow::Next
                            | crate::vm::ControlFlow::Value(_)
                            | crate::vm::ControlFlow::Redo { .. }
                            | crate::vm::ControlFlow::Retry { .. }
                            | crate::vm::ControlFlow::Continue { .. } => {}
                            crate::vm::ControlFlow::Break { value, .. } => {
                                return Ok(Some(value));
                            }
                            crate::vm::ControlFlow::Return { value, position } => {
                                return Err(MetorexError::NonLocalReturn {
                                    value,
                                    location: crate::vm::utils::position_to_location(position),
                                    home_frame: block.home_frame,
                                });
                            }
                        }
                        let stepped = self.send_to_object(
                            Object::string(current.clone()),
                            "succ",
                            vec![],
                            position,
                        )?;
                        match stepped.view() {
                            View::String(text) => current = text.as_str().to_string(),
                            _ => return Ok(Some(receiver.clone())),
                        }
                    }
                }

                // The values are walked the same way `to_a` collects them,
                // so a String range follows `succ` too.
                let elements = self.range_elements(start, end, exclusive, position)?;
                for element in elements {
                    match self.execute_block_with_control_flow(block, vec![element], position)? {
                        crate::vm::ControlFlow::Next
                        | crate::vm::ControlFlow::Value(_)
                        | crate::vm::ControlFlow::Redo { .. }
                        | crate::vm::ControlFlow::Retry { .. }
                        | crate::vm::ControlFlow::Continue { .. } => continue,
                        // `break` ends the walk and answers what it carried,
                        // which is what the call reports.
                        crate::vm::ControlFlow::Break { value, .. } => {
                            return Ok(Some(value));
                        }
                        crate::vm::ControlFlow::Return { value, position } => {
                            return Err(MetorexError::NonLocalReturn {
                                value,
                                location: crate::vm::utils::position_to_location(position),
                                home_frame: block.home_frame,
                            });
                        }
                    }
                }
                Ok(Some(receiver.clone()))
            }
            "to_a" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let elements = self.range_elements(start, end, exclusive, position)?;
                Ok(Some(Object::array(elements)))
            }
            // `size` counts a numeric range without walking it, and answers
            // Infinity when it has no end.
            "size" | "length" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                // Only a range that steps from an Integer has a size. A start
                // that is not a number at all has none to report, while a
                // numeric one that cannot be stepped from is an error.
                if !matches!(
                    start.view(),
                    View::Int(_) | View::BigInt(_) | View::Float(_) | View::Nil
                ) {
                    // A start that has a successor can be walked, it just has
                    // no size to count. One that has none cannot be walked at
                    // all.
                    if matches!(start.view(), View::String(_) | View::Symbol(_))
                        || self.responds_to(start, "succ")
                    {
                        return Ok(Some(Object::Nil));
                    }
                    let message = format!(
                        "can't iterate from {}",
                        self.builtins().class_of(start).ruby_name()
                    );
                    return Err(MetorexError::UncaughtException {
                        exception: Object::exception("TypeError", message.clone()),
                        location: position_to_location(position),
                        message,
                    });
                }
                if !matches!(start.view(), View::Int(_) | View::BigInt(_)) {
                    let message = format!(
                        "can't iterate from {}",
                        self.builtins().class_of(start).ruby_name()
                    );
                    return Err(MetorexError::UncaughtException {
                        exception: Object::exception("TypeError", message.clone()),
                        location: position_to_location(position),
                        message,
                    });
                }
                let infinite = matches!(end.view(), View::Nil)
                    || matches!(end.view(), View::Float(value) if value.is_infinite());
                if infinite {
                    return Ok(Some(Object::Float(f64::INFINITY)));
                }
                let (Some(first), Some(last)) = (numeric_floor(start), numeric_ceiling(end)) else {
                    return Ok(Some(Object::Nil));
                };
                let last = if exclusive && end_is_whole(end) {
                    last - 1
                } else {
                    last
                };
                Ok(Some(Object::Int((last - first + 1).max(0))))
            }
            // A range over numbers or times answers by comparing against its
            // ends, a range of Strings by walking them as `upto` does, and
            // any other by walking its values, which a range missing an end
            // cannot do.
            "member?" | "include?" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let value = &arguments[0];
                let open = |held: &Object| matches!(held.view(), View::Nil);
                let linear = self.ordered_linearly(start)
                    || self.ordered_linearly(end)
                    || (open(start) && open(end) && self.ordered_linearly(value))
                    || self.responds_to(start, "to_int")
                    || self.responds_to(end, "to_int");
                if linear {
                    let covered = self.range_covers(start, end, exclusive, value, position)?;
                    return Ok(Some(Object::Bool(covered)));
                }
                if let (View::String(low), View::String(high)) = (start.view(), end.view()) {
                    let held = match value.view() {
                        View::String(held) => Some(Rc::clone(held)),
                        View::Nil => None,
                        _ if self.responds_to(value, "to_str") => (match self
                            .send_to_object(value.clone(), "to_str", Vec::new(), position)?
                            .view()
                        {
                            View::String(held) => Some(held),
                            converted => {
                                let source = self.builtins().class_of(value).ruby_name();
                                let message = format!(
                                    "can't convert {source} to String ({source}#to_str gives {})",
                                    self.builtins().class_of(&converted.to_object()).ruby_name()
                                );
                                return Err(crate::vm::errors::simple_exception(
                                    "TypeError",
                                    &message,
                                    position,
                                ));
                            }
                        })
                        .cloned(),
                        _ => None,
                    };
                    let Some(held) = held else {
                        return Ok(Some(Object::Bool(false)));
                    };
                    let found = crate::vm::native_methods::string_sets::upto_sequence(
                        &low.as_str(),
                        &high.as_str(),
                        exclusive,
                    )
                    .into_iter()
                    .any(|step| *step == *held.as_str());
                    return Ok(Some(Object::Bool(found)));
                }
                if open(start) || open(end) {
                    return Err(crate::vm::errors::simple_exception(
                        "TypeError",
                        "cannot determine inclusion in beginless/endless ranges",
                        position,
                    ));
                }
                let elements = self.range_elements(start, end, exclusive, position)?;
                for element in elements {
                    if self.elements_equal(&element, value, position)? {
                        return Ok(Some(Object::Bool(true)));
                    }
                }
                Ok(Some(Object::Bool(false)))
            }
            _ => Ok(None),
        }
    }
    /// Whether a value orders along a line, as a number or a Time does,
    /// which Ruby's `linear_object_p` decides.
    fn ordered_linearly(&self, value: &Object) -> bool {
        match value.view() {
            View::Int(_) | View::BigInt(_) | View::Float(_) => true,
            View::Nil | View::String(_) | View::Symbol(_) => false,
            _ => {
                let class = self.builtins().class_of(value);
                crate::vm::method_invocation::descends_from(&class, "Numeric")
                    || crate::vm::method_invocation::descends_from(&class, "Time")
            }
        }
    }
}
