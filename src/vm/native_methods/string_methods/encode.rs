// Writing a string out in another encoding.

use super::*;
use crate::object::{OptionView, View};

impl VirtualMachine {
    /// Writing a string out in another encoding.
    pub(crate) fn call_string_encode_method(
        &mut self,
        _receiver: &Object,
        string_value: &Rc<crate::object::StringValue>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
            // `encode` reads the string's bytes in the encoding they are
            // written in and writes the characters out in the one asked for.
            "encode" => {
                let (arguments, options) = encode_options(arguments);
                // With no encoding named, the string is written in the
                // default internal encoding, or kept in its own.
                let wanted = match arguments.first() {
                    Some(named) => self.encode_target_name(string_value, named, position)?,
                    None => match self.globals().get("__Encoding_default_internal").view() {
                        Some(View::Class(internal)) => internal.name().to_string(),
                        _ => string_value.encoding_name(),
                    },
                };
                // An encoding that spells a character in more than one byte
                // is carried as the bytes themselves, since they spell
                // nothing the text model can hold. A second name says what
                // the bytes are to be read as rather than what the string
                // says it is written in.
                let held = match arguments.get(1) {
                    Some(source) => self
                        .encoding_name_argument(source, position)
                        .unwrap_or_else(|_| string_value.encoding_name()),
                    None => string_value.encoding_name(),
                };
                let replacement = options
                    .replace
                    .clone()
                    .unwrap_or_else(|| default_replacement(&wanted));
                // Between two names for the same encoding nothing is
                // converted, and `invalid: :replace` scrubs what the bytes do
                // not spell.
                let xml = match &options.xml {
                    Some(named) => Some(xml_escape(named, position)?),
                    None => None,
                };
                let newline = newline_conversion(&options, position)?;
                if held == wanted && xml.is_none() && newline.is_none() {
                    // A source named for the bytes reads them as that
                    // encoding, the way `force_encoding` does.
                    let copy = if arguments.len() > 1 {
                        let made = crate::object::StringValue::binary(binary_bytes(string_value));
                        made.retag(wanted.clone());
                        Object::String(Rc::new(made))
                    } else {
                        relabeled_copy(string_value, &wanted)
                    };
                    if let (true, View::String(copied)) = (options.invalid_replace, copy.view()) {
                        let stands_in = options.replace.map_or(Object::Nil, Object::string);
                        return self
                            .scrubbed_string(copied, &[stands_in], position)
                            .map(Some);
                    }
                    return Ok(Some(copy));
                }
                if self.encoding_without_converter(&held, position)?
                    || self.encoding_without_converter(&wanted, position)?
                {
                    // Text that is all ASCII reads the same in every one of
                    // these that is not a dummy, so it needs no converter.
                    let ascii_only = binary_bytes(string_value).is_ascii();
                    if ascii_only && !dummy_encoding(&held) && !dummy_encoding(&wanted) {
                        return Ok(Some(relabeled_copy(string_value, &wanted)));
                    }
                    return Err(converter_not_found(&held, &wanted, position));
                }
                let reads_bytes = string_value.holds_bytes() || arguments.len() > 1;
                let reading = match wide_encoding(&held) {
                    Some(shape) => {
                        let mut read = String::new();
                        for piece in wide_pieces(&binary_bytes(string_value), shape) {
                            match piece {
                                WidePiece::Character(character) => read.push(character),
                                WidePiece::Broken { .. } if options.invalid_replace => {
                                    read.push_str(&replacement)
                                }
                                WidePiece::Broken {
                                    bytes,
                                    after,
                                    incomplete,
                                } => {
                                    let error = invalid_bytes_error(
                                        &bytes, after, incomplete, &held, position,
                                    );
                                    return Err(self.with_conversion_ends(error, &held, &wanted));
                                }
                            }
                        }
                        read
                    }
                    None if held == "ASCII-8BIT" => binary_text(
                        &binary_bytes(string_value),
                        &wanted,
                        options.undef_replace.then_some(replacement.as_str()),
                        position,
                    )
                    .map_err(|error| self.with_conversion_ends(error, "ASCII-8BIT", "UTF-8"))?,
                    None if held == "UTF-8" => self
                        .utf8_text(
                            &binary_bytes(string_value),
                            &options,
                            &replacement,
                            position,
                        )
                        .map_err(|error| self.with_conversion_ends(error, "UTF-8", &wanted))?,
                    None if held == "ISO-2022-JP" => {
                        self.iso_2022_jp_reading(string_value, &wanted, &options, &replacement, position)?
                    }
                    None if spells_shift_jis(&held) && reads_bytes => {
                        let bytes = binary_bytes(string_value);
                        if let Some((broken, after, incomplete)) = shift_jis_problem(&bytes)
                            && !options.invalid_replace
                        {
                            let error =
                                invalid_bytes_error(&broken, after, incomplete, &held, position);
                            return Err(self.with_conversion_ends(error, &held, &wanted));
                        }
                        crate::vm::native_methods::shift_jis_table::shift_jis_text(&bytes)
                    }
                    None if reads_bytes
                        && crate::vm::native_methods::double_byte::double_byte(&held).is_some() =>
                    {
                        let codec =
                            crate::vm::native_methods::double_byte::double_byte(&held).unwrap();
                        let bytes = binary_bytes(string_value);
                        if let Some((broken, after, incomplete)) = codec.problem(&bytes)
                            && !options.invalid_replace
                        {
                            let error =
                                invalid_bytes_error(&broken, after, incomplete, &held, position);
                            return Err(self.with_conversion_ends(error, &held, &wanted));
                        }
                        if !options.undef_replace
                            && let Some(undefined) = codec.undefined(&bytes)
                        {
                            return Err(self.with_conversion_ends(
                                undefined_bytes_error(&undefined, &held, &wanted, position),
                                &held,
                                "UTF-8",
                            ));
                        }
                        codec.text_replacing(
                            &bytes,
                            options.undef_replace.then_some(replacement.as_str()),
                        )
                    }
                    None if held == "EUC-JP" && reads_bytes => {
                        let bytes = binary_bytes(string_value);
                        if !options.invalid_replace
                            && let Some((broken, after, incomplete)) = euc_jp_problem(&bytes)
                        {
                            let error =
                                invalid_bytes_error(&broken, after, incomplete, &held, position);
                            return Err(self.with_conversion_ends(error, &held, &wanted));
                        }
                        if !options.undef_replace
                            && let Some(undefined) = euc_jp_undefined(&bytes)
                        {
                            let message = if wanted == "UTF-8" {
                                format!("{} from EUC-JP to UTF-8", dumped_bytes(&undefined))
                            } else {
                                format!(
                                    "{} to UTF-8 in conversion from EUC-JP to UTF-8 to {wanted}",
                                    dumped_bytes(&undefined)
                                )
                            };
                            let error = undefined_conversion(&message, &undefined, "EUC-JP", position);
                            return Err(self.with_conversion_ends(error, "EUC-JP", "UTF-8"));
                        }
                        euc_jp_text_replacing(&bytes, &replacement)
                    }
                    None if held == "US-ASCII" => {
                        let bytes = binary_bytes(string_value);
                        match bytes.iter().find(|byte| !byte.is_ascii()) {
                            Some(byte) if !options.invalid_replace => {
                                let error =
                                    invalid_bytes_error(&[*byte], None, false, &held, position);
                                return Err(self.with_conversion_ends(error, &held, &wanted));
                            }
                            _ => bytes
                                .iter()
                                .map(|byte| match byte.is_ascii() {
                                    true => (*byte as char).to_string(),
                                    false => replacement.clone(),
                                })
                                .collect(),
                        }
                    }
                    None => {
                        let bytes = binary_bytes(string_value);
                        match undefined_single_byte(&bytes, &held) {
                            Some(_) if reads_bytes && options.undef_replace => {
                                latin_text_replacing(&bytes, &held, &replacement)
                                    .unwrap_or_else(|| string_value.to_text())
                            }
                            Some(byte) if reads_bytes => {
                                return Err(self.with_conversion_ends(
                                    undefined_bytes_error(&[byte], &held, &wanted, position),
                                    &held,
                                    "UTF-8",
                                ));
                            }
                            _ => match latin_text(&bytes, &held) {
                                Some(spelled) if reads_bytes => spelled,
                                _ => string_value.to_text(),
                            },
                        }
                    }
                };
                // `xml:` writes a character the destination cannot spell as a
                // character reference. Failing that, `undef: :replace` stands
                // the replacement in for it, and failing that `fallback:` is
                // asked for one.
                let reading = match newline {
                    Some(NewlineConversion::Universal) => {
                        reading.replace("\r\n", "\n").replace('\r', "\n")
                    }
                    _ => reading,
                };
                let reading = if let Some(escape) = xml {
                    xml_escaped(&reading, escape, &wanted)
                } else if options.undef_replace {
                    reading
                        .chars()
                        .map(|character| match destination_spells(&wanted, character) {
                            true => character.to_string(),
                            false => replacement.clone(),
                        })
                        .collect()
                } else if let Some(fallback) = &options.fallback {
                    self.fallback_text(reading, fallback, &wanted, position)?
                } else {
                    reading
                };
                let reading = match newline {
                    Some(NewlineConversion::Crlf) => reading.replace('\n', "\r\n"),
                    Some(NewlineConversion::Cr) => reading.replace('\n', "\r"),
                    _ => reading,
                };
                if wanted == "US-ASCII"
                    && let Some(character) = reading.chars().find(|character| !character.is_ascii())
                {
                    return Err(
                        self.undefined_conversion_error(character, &held, &wanted, position)
                    );
                }
                // ISO-2022-JP writes its Japanese runs between escapes, so
                // the bytes are built rather than mapped one for one.
                if wanted == "ISO-2022-JP" {
                    let bytes =
                        crate::vm::native_methods::euc_jp_table::iso_2022_jp_bytes(&reading)
                            .map_err(|character| {
                                self.undefined_conversion_error(character, &held, &wanted, position)
                            })?;
                    let made = crate::object::StringValue::binary(bytes);
                    made.set_encoding(wanted);
                    return Ok(Some(Object::String(Rc::new(made))));
                }
                if spells_shift_jis(&wanted) {
                    let bytes =
                        crate::vm::native_methods::shift_jis_table::shift_jis_bytes(&reading)
                            .map_err(|character| {
                                self.undefined_conversion_error(character, &held, &wanted, position)
                            })?;
                    let made = crate::object::StringValue::binary(bytes);
                    made.set_encoding(wanted);
                    return Ok(Some(Object::String(Rc::new(made))));
                }
                if let Some(codec) = crate::vm::native_methods::double_byte::double_byte(&wanted) {
                    let bytes = codec.bytes(&reading).map_err(
                        |character| {
                            self.undefined_conversion_error(character, &held, &wanted, position)
                        },
                    )?;
                    let made = crate::object::StringValue::binary(bytes);
                    made.set_encoding(wanted);
                    return Ok(Some(Object::String(Rc::new(made))));
                }
                if wanted == "EUC-JP" {
                    let bytes = crate::vm::native_methods::euc_jp_table::euc_jp_bytes(&reading)
                        .map_err(|character| {
                            self.undefined_conversion_error(character, &held, &wanted, position)
                        })?;
                    let made = crate::object::StringValue::binary(bytes);
                    made.set_encoding(wanted);
                    return Ok(Some(Object::String(Rc::new(made))));
                }
                if let Some(spelled) = latin_bytes(&reading, &wanted) {
                    let bytes = spelled.map_err(|character| {
                        self.undefined_conversion_error(character, &held, &wanted, position)
                    })?;
                    let made = crate::object::StringValue::binary(bytes);
                    made.set_encoding(wanted);
                    return Ok(Some(Object::String(Rc::new(made))));
                }
                if let Some(shape) = wide_encoding(&wanted) {
                    let made = Object::String(Rc::new(crate::object::StringValue::binary(
                        wide_bytes(&reading, shape),
                    )));
                    if let View::String(copied) = made.view() {
                        copied.set_encoding(wanted);
                    }
                    return Ok(Some(made));
                }
                let copy = Object::string(reading);
                if let View::String(copied) = copy.view() {
                    copied.set_encoding(wanted);
                }
                Ok(Some(copy))
            }
            _ => Ok(None),
        }
    }

    /// The text ISO-2022-JP bytes spell, on the way to `wanted`. Bytes that
    /// spell nothing, and pairs JIS X 0208 has no character for, raise unless
    /// the options replace them.
    fn iso_2022_jp_reading(
        &mut self,
        string_value: &crate::object::StringValue,
        wanted: &str,
        options: &EncodeOptions,
        replacement: &str,
        position: Position,
    ) -> Result<String, MetorexError> {
        use crate::vm::native_methods::iso_2022_jp::{JisPiece, iso_2022_jp_pieces};
        let mut read = String::new();
        for piece in iso_2022_jp_pieces(&binary_bytes(string_value)) {
            match piece {
                JisPiece::Character(character) => read.push(character),
                JisPiece::Broken { .. } if options.invalid_replace => read.push_str(replacement),
                JisPiece::Broken {
                    bytes,
                    after,
                    incomplete,
                } => {
                    let error =
                        invalid_bytes_error(&bytes, after, incomplete, "ISO-2022-JP", position);
                    return Err(self.with_conversion_ends(
                        error,
                        "ISO-2022-JP",
                        "stateless-ISO-2022-JP",
                    ));
                }
                JisPiece::Undefined(_) if options.undef_replace => read.push_str(replacement),
                JisPiece::Undefined(bytes) => {
                    let onward = if wanted == "UTF-8" {
                        String::new()
                    } else {
                        format!(" to {wanted}")
                    };
                    let message = format!(
                        "{} to UTF-8 in conversion from ISO-2022-JP to stateless-ISO-2022-JP to EUC-JP to UTF-8{onward}",
                        dumped_bytes(&bytes)
                    );
                    let error = undefined_conversion(&message, &bytes, "EUC-JP", position);
                    return Err(self.with_conversion_ends(error, "EUC-JP", "UTF-8"));
                }
            }
        }
        Ok(read)
    }

}
