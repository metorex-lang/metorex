// The File class methods.

use super::*;

impl VirtualMachine {
    /// The File class methods, which read and write through the operating
    /// system rather than through an instance.
    pub(crate) fn call_file_class_methods(
        &mut self,
        class_rc: &Rc<Class>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<ClassMethodAnswer, MetorexError> {
        // Minimal File class methods used by mspec's `fixture` helper.
        if class_rc.name() == "File" {
            match method_name {
                // Every part of a path but the last, as many times over as
                // the level says.
                "dirname" => {
                    if arguments.is_empty() || arguments.len() > 2 {
                        return Err(crate::vm::errors::argument_count_error(
                            crate::vm::errors::Arity::Range(1, 2),
                            arguments.len(),
                            position,
                        ));
                    }
                    let named = self.path_name_argument("dirname", &arguments[0], position)?;
                    let encoding = match &arguments[0] {
                        Object::String(held) => held.encoding_name(),
                        _ => "UTF-8".to_string(),
                    };
                    if let Object::String(held) = &arguments[0]
                        && !crate::vm::native_methods::string_methods::encoding_is_ascii_compatible(
                            &held.encoding_name(),
                        )
                    {
                        let message = format!(
                            "path name must be ASCII-compatible ({}): {:?}",
                            held.encoding_name(),
                            held.as_str()
                        );
                        return Err(crate::vm::errors::simple_exception(
                            "Encoding::CompatibilityError",
                            &message,
                            position,
                        ));
                    }
                    let mut level = match arguments.get(1) {
                        None => 1_i64,
                        Some(held) => self.integer_argument("dirname", held, position)?,
                    };
                    if level < 0 {
                        let message = format!("negative level: {}", level);
                        return Err(crate::vm::errors::simple_exception(
                            "ArgumentError",
                            &message,
                            position,
                        ));
                    }
                    let mut held = named;
                    while level > 0 {
                        let stepped = parent_of_path(&held);
                        if stepped == held {
                            break;
                        }
                        held = stepped;
                        level -= 1;
                    }
                    return Ok(Answered(Object::String(Rc::new(
                        crate::object::StringValue::with_encoding(held, encoding),
                    ))));
                }
                // The last part of a path, with a suffix taken off when one
                // is named. `".*"` means whatever extension the name carries.
                "basename" => {
                    if arguments.is_empty() || arguments.len() > 2 {
                        return Err(crate::vm::errors::argument_count_error(
                            crate::vm::errors::Arity::Range(1, 2),
                            arguments.len(),
                            position,
                        ));
                    }
                    // A name written in an encoding that spells the ASCII
                    // letters in more than one byte cannot be read as a path.
                    if let Object::String(held) = &arguments[0]
                        && !crate::vm::native_methods::string_methods::encoding_is_ascii_compatible(
                            &held.encoding_name(),
                        )
                    {
                        let message = format!(
                            "path name must be ASCII-compatible ({}): {:?}",
                            held.encoding_name(),
                            held.as_str()
                        );
                        return Err(crate::vm::errors::simple_exception(
                            "Encoding::CompatibilityError",
                            &message,
                            position,
                        ));
                    }
                    // A suffix is named by text, and nothing else stands for
                    // one.
                    if let Some(held) = arguments.get(1)
                        && !matches!(held, Object::String(_))
                        && !self.answers_to(held, "to_str", position)?
                    {
                        return Err(method_argument_type_error(
                            "basename", "String", held, position,
                        ));
                    }
                    let path = self.path_name_argument("basename", &arguments[0], position)?;
                    let trimmed = path.trim_end_matches('/');
                    let held = if trimmed.is_empty() {
                        if path.is_empty() { "" } else { "/" }
                    } else {
                        trimmed.rsplit('/').next().unwrap_or(trimmed)
                    };
                    let mut name = held.to_string();
                    if let Some(Object::String(suffix)) = arguments.get(1) {
                        let suffix = suffix.to_text();
                        if suffix == ".*" {
                            if let Some(dot) = name.rfind('.')
                                && dot > 0
                            {
                                name.truncate(dot);
                            }
                        } else if name.len() > suffix.len() && name.ends_with(&*suffix) {
                            name.truncate(name.len() - suffix.len());
                        }
                    }
                    return Ok(Answered(path_piece(name, &arguments[0])));
                }
                // The extension a name carries, and the empty string for a
                // name that carries none.
                "extname" => {
                    if arguments.len() != 1 {
                        return Err(method_argument_error(
                            method_name,
                            1,
                            arguments.len(),
                            position,
                        ));
                    }
                    let path = self.path_name_argument("extname", &arguments[0], position)?;
                    let held = path.trim_end_matches('/').rsplit('/').next().unwrap_or("");
                    // A dot at the front of a name is part of the name rather
                    // than the start of an extension, so a name of nothing but
                    // dots has no extension at all. A name that ends in a dot
                    // carries that dot as its extension.
                    let stripped = held.trim_start_matches('.');
                    let leading = held.len() - stripped.len();
                    let found = match stripped.rfind('.') {
                        Some(dot) if dot + 1 == stripped.len() => ".",
                        Some(dot) => &held[leading + dot..],
                        None => "",
                    };
                    return Ok(Answered(path_piece(found.to_string(), &arguments[0])));
                }
                // The directory and the name a path is made of, as a pair.
                "split" => {
                    if arguments.len() != 1 {
                        return Err(method_argument_error(
                            method_name,
                            1,
                            arguments.len(),
                            position,
                        ));
                    }
                    let path = self.path_name_argument("split", &arguments[0], position)?;
                    // The pieces keep the encoding the whole path was in.
                    let whole = match &arguments[0] {
                        held @ Object::String(_) => held.clone(),
                        _ => Object::string(path),
                    };
                    let arguments = &[whole];
                    let directory = self
                        .call_class_methods(class_rc, "dirname", arguments, position)?
                        .unwrap_or(Object::Nil);
                    let name = self
                        .call_class_methods(class_rc, "basename", arguments, position)?
                        .unwrap_or(Object::Nil);
                    return Ok(Answered(Object::array(vec![directory, name])));
                }
                "realpath" => {
                    if let Some(Object::String(s)) = arguments.first() {
                        let expanded = std::fs::canonicalize(&*s.as_str())
                            .ok()
                            .and_then(|p| p.to_str().map(String::from))
                            .unwrap_or_else(|| s.as_str().to_string());
                        return Ok(Answered(Object::string(expanded)));
                    }
                }
                _ => {}
            }
        }
        Ok(Unclaimed)
    }
}

/// A piece of a path, written in the same encoding the whole path was.
fn path_piece(piece: String, whole: &Object) -> Object {
    let made = Object::string(piece);
    if let (Object::String(held), Object::String(written)) = (whole, &made) {
        written.set_encoding(held.encoding_name());
        if held.holds_bytes() {
            written.mark_bytes();
        }
    }
    made
}
