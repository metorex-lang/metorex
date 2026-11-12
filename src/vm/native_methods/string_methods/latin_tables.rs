// Reading and writing the single-byte encodings, through the tables
// scripts/generate_single_byte_tables.rb writes from MRI.

use super::single_byte_tables::{SINGLE_BYTE_TABLES, UNDEFINED};

/// The code point each byte stands for in the single-byte encoding `named`.
fn table_for(named: &str) -> Option<&'static [u32; 256]> {
    SINGLE_BYTE_TABLES
        .iter()
        .find(|(name, _)| *name == named)
        .map(|(_, table)| table)
}

/// The text a run of bytes spells in a single-byte encoding. A byte the
/// encoding leaves undefined reads as the Latin-1 character of its number,
/// which `undefined_single_byte` lets a converter refuse first.
pub(crate) fn latin_text(bytes: &[u8], named: &str) -> Option<String> {
    let table = table_for(named)?;
    Some(
        bytes
            .iter()
            .map(|byte| {
                let point = table[*byte as usize];
                if point == UNDEFINED {
                    *byte as char
                } else {
                    char::from_u32(point).unwrap_or(*byte as char)
                }
            })
            .collect(),
    )
}

/// The text a run of bytes spells in a single-byte encoding, with
/// `replacement` standing in for each byte the encoding leaves undefined.
pub(crate) fn latin_text_replacing(bytes: &[u8], named: &str, replacement: &str) -> Option<String> {
    let table = table_for(named)?;
    let mut text = String::with_capacity(bytes.len());
    for byte in bytes {
        match char::from_u32(table[*byte as usize]) {
            Some(spelled) => text.push(spelled),
            None => text.push_str(replacement),
        }
    }
    Some(text)
}

/// Bytes spelled as `String#dump` spells them, quotes and all, which is how
/// a conversion error names the bytes it stopped at.
pub(crate) fn dumped_bytes(bytes: &[u8]) -> String {
    let mut written = String::from("\"");
    for (at, byte) in bytes.iter().enumerate() {
        match byte {
            b'"' | b'\\' => {
                written.push('\\');
                written.push(char::from(*byte));
            }
            b'#' if matches!(bytes.get(at + 1), Some(b'{' | b'$' | b'@')) => {
                written.push_str("\\#")
            }
            b'\n' => written.push_str("\\n"),
            b'\r' => written.push_str("\\r"),
            b'\t' => written.push_str("\\t"),
            0x0c => written.push_str("\\f"),
            0x0b => written.push_str("\\v"),
            0x08 => written.push_str("\\b"),
            0x07 => written.push_str("\\a"),
            0x1b => written.push_str("\\e"),
            0x20..=0x7e => written.push(char::from(*byte)),
            _ => written.push_str(&format!("\\x{byte:02X}")),
        }
    }
    written.push('"');
    written
}

/// The error a character the source encoding leaves undefined raises, naming
/// the step to UTF-8 the conversion stops at.
pub(crate) fn undefined_bytes_error(
    bytes: &[u8],
    source: &str,
    wanted: &str,
    position: crate::lexer::Position,
) -> crate::error::MetorexError {
    let spelled = dumped_bytes(bytes);
    let message = if wanted == "UTF-8" {
        format!("{spelled} from {source} to UTF-8")
    } else {
        format!("{spelled} to UTF-8 in conversion from {source} to UTF-8 to {wanted}")
    };
    undefined_conversion(&message, bytes, source, position)
}

/// An UndefinedConversionError with `message`, whose `error_char` holds
/// `bytes` in the encoding `tagged`.
pub(crate) fn undefined_conversion(
    message: &str,
    bytes: &[u8],
    tagged: &str,
    position: crate::lexer::Position,
) -> crate::error::MetorexError {
    let error = crate::vm::errors::simple_exception(
        "Encoding::UndefinedConversionError",
        message,
        position,
    );
    let character = crate::object::StringValue::binary(bytes.to_vec());
    character.retag(tagged.to_string());
    super::set_exception_attribute(
        &error,
        "error_char",
        crate::object::Object::String(std::rc::Rc::new(character)),
    );
    error
}

/// The first byte of a run that a single-byte encoding leaves undefined.
pub(crate) fn undefined_single_byte(bytes: &[u8], named: &str) -> Option<u8> {
    let table = table_for(named)?;
    bytes
        .iter()
        .copied()
        .find(|byte| table[*byte as usize] == UNDEFINED)
}

/// The bytes text spells in a single-byte encoding, or the first character
/// the encoding has no byte for.
pub(crate) fn latin_bytes(text: &str, named: &str) -> Option<Result<Vec<u8>, char>> {
    let table = table_for(named)?;
    let mut bytes = Vec::with_capacity(text.len());
    for character in text.chars() {
        let point = character as u32;
        // The byte of the same number first, which every encoding keeping
        // ASCII has for ASCII, then the first byte that stands for it.
        let found = if point < 256 && table[point as usize] == point {
            Some(point as u8)
        } else {
            table
                .iter()
                .position(|held| *held == point)
                .map(|at| at as u8)
        };
        match found {
            Some(byte) => bytes.push(byte),
            None => return Some(Err(character)),
        }
    }
    Some(Ok(bytes))
}
