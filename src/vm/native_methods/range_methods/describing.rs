// What a range reads back as, and whether two of them are the same.

use super::*;
use crate::object::View;

impl VirtualMachine {
    /// What a range reads back as, and whether two of them are the same.
    pub(crate) fn call_range_describing_method(
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
            "to_set" => {
                if matches!(end.view(), View::Nil) {
                    return Err(crate::vm::errors::simple_exception(
                        "RangeError",
                        "cannot convert endless range to a set",
                        position,
                    ));
                }
                let elements = self.range_elements(start, end, exclusive, position)?;
                let walked = Object::array(elements);
                self.send_to_object(walked, "to_set", arguments.to_vec(), position)
                    .map(Some)
            }
            "exclude_end?" => Ok(Some(Object::Bool(exclusive))),
            // The endpoints render through their own `inspect`, and an
            // endless or beginless range leaves its side out.
            "to_s" | "inspect" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let separator = if exclusive { "..." } else { ".." };
                if method_name == "to_s" {
                    let first = self.interpolated_text(start, position)?;
                    let last = self.interpolated_text(end, position)?;
                    return Ok(Some(Object::string(format!("{first}{separator}{last}"))));
                }
                // A range with neither end shows both nils, where one open
                // side alone shows nothing.
                let both_open =
                    matches!(start.view(), View::Nil) && matches!(end.view(), View::Nil);
                let first = match start.view() {
                    View::Nil if !both_open => String::new(),
                    _ => self.get_inspect_representation(start, position)?,
                };
                let last = match end.view() {
                    View::Nil if !both_open => String::new(),
                    _ => self.get_inspect_representation(end, position)?,
                };
                Ok(Some(Object::string(format!(
                    "{}{}{}",
                    first, separator, last
                ))))
            }
            // Two ranges are equal when their ends and their exclusivity
            // match. `eql?` compares the ends with `eql?` rather than `==`.
            "==" | "eql?" => {
                let Some(other) = arguments.first() else {
                    return Err(method_argument_error(method_name, 1, 0, position));
                };
                let View::Range {
                    start: other_start,
                    end: other_end,
                    exclusive: other_exclusive,
                    ..
                } = other.view()
                else {
                    return Ok(Some(Object::Bool(false)));
                };
                if exclusive != other_exclusive {
                    return Ok(Some(Object::Bool(false)));
                }
                let same = if method_name == "eql?" {
                    self.values_eql(start, other_start, position)?
                        && self.values_eql(end, other_end, position)?
                } else {
                    self.elements_equal(start, other_start, position)?
                        && self.elements_equal(end, other_end, position)?
                };
                Ok(Some(Object::Bool(same)))
            }
            "count" => {
                if !arguments.is_empty() || self.pending_block.is_some() {
                    return Ok(None);
                }
                // A range with no beginning or no end holds endlessly many
                // values, which Ruby reports as Infinity.
                if matches!(start.view(), View::Nil) || matches!(end.view(), View::Nil) {
                    return Ok(Some(Object::Float(f64::INFINITY)));
                }
                let elements = self.range_elements(start, end, exclusive, position)?;
                Ok(Some(Object::Int(elements.len() as i64)))
            }
            _ => Ok(None),
        }
    }
}
