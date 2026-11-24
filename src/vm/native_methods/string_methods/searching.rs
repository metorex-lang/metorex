// Whether a string holds, opens with, or closes on another.

use super::*;
use crate::object::View;

impl VirtualMachine {
    /// Whether a string holds, opens with, or closes on another.
    pub(crate) fn call_string_searching_method(
        &mut self,
        _receiver: &Object,
        string_value: &Rc<crate::object::StringValue>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
            "include?" | "contains?" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                // Text written in an encoding this string cannot be read
                // alongside is refused rather than searched for.
                if let View::String(other) = arguments[0].view()
                    && !strings_comparable(string_value, other)
                {
                    return Err(clashing_encodings_error(string_value, other, position));
                }
                // Anything that spells itself as text is read as the text it
                // spells, which is how a wrapper around a String is searched
                // for.
                let wanted = self.string_argument(method_name, &arguments[0], position)?;
                // Text that spells no characters is never found, as Ruby
                // never finds it.
                let held = match (broken_utf8_reading(string_value), arguments[0].view()) {
                    (_, View::String(other)) if broken_utf8_reading(other).is_some() => false,
                    (Some(read), _) => read.contains(wanted.as_str()),
                    (None, _) => string_value.as_str().contains(wanted.as_str()),
                };
                Ok(Some(Object::Bool(held)))
            }
            "start_with?" | "end_with?" => {
                // With no arguments nothing matches, which is what Ruby
                // answers rather than refusing the call.
                let mut result = false;
                for arg in arguments {
                    // An argument that is not a String is asked for one, which
                    // is what `to_str` answers.
                    let arg = &match arg.view() {
                        View::String(_) => arg.clone(),
                        _ if self.responds_to(arg, "to_str") => {
                            self.send_to_object(arg.clone(), "to_str", vec![], position)?
                        }
                        _ => arg.clone(),
                    };
                    match arg.view() {
                        View::String(s) => {
                            // Text written in an encoding this string cannot
                            // be read alongside is refused rather than
                            // compared.
                            if !strings_comparable(string_value, s) {
                                return Err(clashing_encodings_error(string_value, s, position));
                            }
                            // A match has to start where a character does, so
                            // the ends are compared by character rather than
                            // by byte. An encoding of more than one byte to a
                            // character is read as characters first.
                            let fits =
                                if let Some(shape) = wide_encoding(string_value.encoding_name()) {
                                    let letters: Vec<char> =
                                        wide_text(&binary_bytes(string_value), shape)
                                            .chars()
                                            .collect();
                                    let wanted = binary_bytes(s);
                                    (1..=letters.len()).any(|count| {
                                        let taken: String = if method_name == "start_with?" {
                                            letters[..count].iter().collect()
                                        } else {
                                            letters[letters.len() - count..].iter().collect()
                                        };
                                        wide_bytes(&taken, shape) == wanted
                                    })
                                } else {
                                    // Text that matches whole characters can
                                    // only start and end on a character's
                                    // edge, so only its length is compared.
                                    let held = searched_text(string_value);
                                    let wanted = searched_text(s);
                                    if method_name == "start_with?" {
                                        held.starts_with(&*wanted)
                                    } else {
                                        held.ends_with(&*wanted)
                                    }
                                };
                            if fits {
                                result = true;
                                break;
                            }
                        }
                        // `start_with?` also takes a Regexp, which matches at
                        // the front of the string and records the match the
                        // way any other match does.
                        View::Regex(pattern, flags) if method_name == "start_with?" => {
                            let anchored = format!("\\A(?:{})", pattern);
                            let subject = string_value.as_str().to_string();
                            let found =
                                self.regexp_match_data(&anchored, flags, &subject, 0, position)?;
                            if found.is_some() {
                                result = true;
                                break;
                            }
                        }
                        _ => {
                            return Err(method_argument_type_error(
                                method_name,
                                "String",
                                arg,
                                position,
                            ));
                        }
                    }
                }
                Ok(Some(Object::Bool(result)))
            }
            "starts_with?" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                match arguments[0].view() {
                    View::String(prefix) => Ok(Some(Object::Bool(
                        string_value.as_str().starts_with(&*prefix.as_str()),
                    ))),
                    _ => Err(method_argument_type_error(
                        method_name,
                        "String",
                        &arguments[0],
                        position,
                    )),
                }
            }
            "ends_with?" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                match arguments[0].view() {
                    View::String(suffix) => Ok(Some(Object::Bool(
                        string_value.as_str().ends_with(&*suffix.as_str()),
                    ))),
                    _ => Err(method_argument_type_error(
                        method_name,
                        "String",
                        &arguments[0],
                        position,
                    )),
                }
            }
            _ => Ok(None),
        }
    }
}

/// The characters a UTF-8 string holding bytes that spell no character
/// reads as when searched: each such byte stands alone as a character no
/// valid text holds. None for a string that is not such a one.
pub(crate) fn broken_utf8_reading(string_value: &crate::object::StringValue) -> Option<String> {
    if string_value.encoding_name() != "UTF-8" || !string_value.reads_bytes() {
        return None;
    }
    let bytes = binary_bytes(string_value);
    let mut read = String::with_capacity(bytes.len());
    let mut rest: &[u8] = &bytes;
    while !rest.is_empty() {
        match std::str::from_utf8(rest) {
            Ok(text) => {
                read.push_str(text);
                break;
            }
            Err(error) => {
                let good = error.valid_up_to();
                read.push_str(std::str::from_utf8(&rest[..good]).unwrap_or_default());
                let bad = error.error_len().unwrap_or(rest.len() - good).max(1);
                for byte in &rest[good..good + bad] {
                    read.push(stand_in_for_byte(*byte));
                }
                rest = &rest[good + bad..];
            }
        }
    }
    Some(read)
}

/// The character a byte that spells no character stands as, from a range
/// of private-use characters set aside for the purpose.
fn stand_in_for_byte(byte: u8) -> char {
    char::from_u32(0x10_FF00 + u32::from(byte)).unwrap_or(char::REPLACEMENT_CHARACTER)
}

/// The text a string is searched as.
pub(crate) fn searched_text(string_value: &crate::object::StringValue) -> String {
    broken_utf8_reading(string_value).unwrap_or_else(|| string_value.as_str().to_string())
}
