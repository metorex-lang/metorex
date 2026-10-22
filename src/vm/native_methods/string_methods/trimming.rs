// Joining strings, and cutting whitespace off either end.

use super::*;
use crate::object::View;

impl VirtualMachine {
    /// Joining strings, and cutting whitespace off either end.
    pub(crate) fn call_string_trimming_method(
        &mut self,
        receiver: &Object,
        string_value: &Rc<crate::object::StringValue>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
            "+" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                // Anything that spells itself as text is joined as the text
                // it spells, which is what `to_str` is asked for.
                let given = match arguments[0].view() {
                    View::String(_) => arguments[0].clone(),
                    _ if self.responds_to(&arguments[0], "to_str") => {
                        self.send_to_object(arguments[0].clone(), "to_str", vec![], position)?
                    }
                    _ => {
                        return Err(crate::vm::errors::simple_exception(
                            "TypeError",
                            &format!(
                                "no implicit conversion of {} into String",
                                crate::vm::errors::conversion_subject(&arguments[0])
                            ),
                            position,
                        ));
                    }
                };
                let View::String(rhs) = given.view() else {
                    return Err(method_argument_type_error(
                        method_name,
                        "String",
                        &arguments[0],
                        position,
                    ));
                };
                // Two strings written in encodings that cannot be read
                // alongside each other are refused, unless one of them is
                // empty, which carries the other's encoding.
                let joined_encoding = if rhs.as_str().is_empty() {
                    string_value.encoding_name()
                } else if string_value.as_str().is_empty() {
                    rhs.encoding_name()
                } else if !strings_comparable(string_value, rhs) {
                    return Err(clashing_encodings_error(string_value, rhs, position));
                } else if string_value.as_str().is_ascii() && !rhs.as_str().is_ascii() {
                    rhs.encoding_name()
                } else {
                    string_value.encoding_name()
                };
                let mut combined = string_value.as_str().to_string();
                combined.push_str(&rhs.as_str());
                let made = Object::string(combined);
                if let View::String(built) = made.view() {
                    built.set_encoding(joined_encoding);
                }
                Ok(Some(made))
            }
            // `lstrip` and `rstrip` trim one end. Ruby counts a NUL as
            // whitespace at the right end, which `trim_end` does not.
            // `each_line` and `lines` split on a separator, keeping it on the
            // end of each piece the way Ruby does.
            "each_line" | "lines" => {
                self.walk_lines(receiver, string_value, method_name, arguments, position)
            }
            // A string whose bytes spell nothing cannot be trimmed from the
            // back. From the front it is trimmed up to its first character
            // that is not a space, and refused only when that one is broken.
            "lstrip" | "lstrip!" if !holds_valid_text(string_value) && arguments.is_empty() => {
                let bytes = binary_bytes(string_value);
                let Some(start) = whole_first_character(string_value, &bytes) else {
                    return Err(broken_text_error(string_value, position));
                };
                let trimmed = bytes[start..].to_vec();
                if method_name == "lstrip!" {
                    if start == 0 {
                        return Ok(Some(Object::Nil));
                    }
                    string_value.replace_bytes(trimmed);
                    return Ok(Some(receiver.clone()));
                }
                let made = crate::object::StringValue::binary(trimmed);
                made.retag(string_value.encoding_name());
                Ok(Some(Object::String(Rc::new(made))))
            }
            "rstrip" | "rstrip!" | "strip" | "strip!" if !holds_valid_text(string_value) => {
                // `strip` trims the front first, which refuses a broken first
                // character before the back is looked at.
                if method_name.starts_with("strip")
                    && whole_first_character(string_value, &binary_bytes(string_value)).is_none()
                {
                    return Err(broken_text_error(string_value, position));
                }
                let message = format!("invalid byte sequence in {}", string_value.encoding_name());
                Err(crate::vm::errors::simple_exception(
                    "Encoding::CompatibilityError",
                    &message,
                    position,
                ))
            }
            "lstrip" => {
                if !arguments.is_empty() {
                    return self
                        .strip_selected(string_value, method_name, arguments, position)
                        .map(Some);
                }
                Ok(Some(Object::string(
                    string_value
                        .as_str()
                        .trim_start_matches(|letter: char| letter.is_whitespace() || letter == '\0')
                        .to_string(),
                )))
            }
            "rstrip" => {
                if !arguments.is_empty() {
                    return self
                        .strip_selected(string_value, method_name, arguments, position)
                        .map(Some);
                }
                Ok(Some(Object::string(
                    string_value
                        .as_str()
                        .trim_end_matches(|letter: char| letter.is_whitespace() || letter == '\0')
                        .to_string(),
                )))
            }
            "reverse" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let mut units = character_units(string_value);
                units.reverse();
                // The reversed string is written in the same encoding.
                let reversed = crate::object::StringValue::with_encoding(
                    units.concat(),
                    string_value.encoding_name(),
                );
                if string_value.holds_bytes() {
                    reversed.mark_bytes();
                }
                Ok(Some(Object::String(Rc::new(reversed))))
            }
            "last" => {
                let chars: Vec<char> = string_value.as_str().chars().collect();
                if chars.is_empty() {
                    Ok(Some(Object::Nil))
                } else {
                    Ok(Some(Object::string(chars.last().unwrap().to_string())))
                }
            }
            _ => Ok(None),
        }
    }
}

/// Where the first character that is not a space starts in a UTF-8 string
/// whose bytes do not all spell characters, or None when that character is
/// broken or the string is in another encoding.
fn whole_first_character(string_value: &crate::object::StringValue, bytes: &[u8]) -> Option<usize> {
    if string_value.encoding_name() != "UTF-8" {
        return None;
    }
    let start = bytes
        .iter()
        .position(|byte| !matches!(byte, b' ' | b'\t'..=b'\r' | 0))
        .unwrap_or(bytes.len());
    // A character is at most four bytes, so the first one is whole when
    // those four begin with one.
    let first = &bytes[start..bytes.len().min(start + 4)];
    let whole = std::str::from_utf8(first).map_or_else(|broken| broken.valid_up_to() > 0, |_| true);
    whole.then_some(start)
}
