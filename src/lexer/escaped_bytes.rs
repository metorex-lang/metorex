//! Bytes that are not UTF-8, carried in source text as characters of their own.

/// Where the characters standing for bytes that are not UTF-8 start. They
/// sit in the private use area, which source text has no other use for.
const ESCAPED_BYTE_BASE: u32 = 0xF700;

/// The character standing for a byte past ASCII that spells no text.
pub fn escaped_byte(byte: u8) -> char {
    char::from_u32(ESCAPED_BYTE_BASE + u32::from(byte)).unwrap_or(char::REPLACEMENT_CHARACTER)
}

/// Source text read from bytes that are not all UTF-8: each run that is
/// stays as written, and each byte that is not becomes an escaped character.
pub fn escaped_source_text(bytes: &[u8]) -> String {
    let mut text = String::new();
    let mut rest = bytes;
    loop {
        match std::str::from_utf8(rest) {
            Ok(valid) => {
                text.push_str(valid);
                return text;
            }
            Err(error) => {
                let (valid, after) = rest.split_at(error.valid_up_to());
                text.push_str(&String::from_utf8_lossy(valid));
                let escaped = char::from_u32(ESCAPED_BYTE_BASE + u32::from(after[0]));
                text.extend(escaped);
                rest = &after[1..];
            }
        }
    }
}

/// The bytes a literal holding escaped characters was written with: each
/// escaped character is its byte, and the rest are their UTF-8. `None`
/// when the literal holds no escaped character.
pub fn escaped_source_bytes(text: &str) -> Option<Vec<u8>> {
    let escaped = |letter: char| {
        (ESCAPED_BYTE_BASE + 0x80..=ESCAPED_BYTE_BASE + 0xFF).contains(&(letter as u32))
    };
    if !text.chars().any(escaped) {
        return None;
    }
    let mut bytes = Vec::new();
    for letter in text.chars() {
        if escaped(letter) {
            bytes.push((letter as u32 - ESCAPED_BYTE_BASE) as u8);
        } else {
            let mut spelled = [0; 4];
            bytes.extend_from_slice(letter.encode_utf8(&mut spelled).as_bytes());
        }
    }
    Some(bytes)
}
