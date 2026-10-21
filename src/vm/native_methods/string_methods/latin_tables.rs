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

/// The error a byte the source encoding leaves undefined raises, naming the
/// step to UTF-8 the conversion stops at.
pub(crate) fn undefined_byte_error(
    byte: u8,
    source: &str,
    wanted: &str,
    position: crate::lexer::Position,
) -> crate::error::MetorexError {
    let spelled = format!("\"\\x{byte:02X}\"");
    let message = if wanted == "UTF-8" {
        format!("{spelled} from {source} to UTF-8")
    } else {
        format!("{spelled} to UTF-8 in conversion from {source} to UTF-8 to {wanted}")
    };
    let error = crate::vm::errors::simple_exception(
        "Encoding::UndefinedConversionError",
        &message,
        position,
    );
    super::set_exception_attribute(
        &error,
        "error_char",
        crate::vm::native_methods::pack_format::bytes_to_string(&[byte]),
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
