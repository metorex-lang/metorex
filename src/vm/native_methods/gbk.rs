//! Reading and writing GBK. A byte from 0x81 to 0xFE opens a two-byte
//! character whose second byte is 0x40 to 0x7E or 0x80 to 0xFE, and every
//! other byte stands alone.

use super::gbk_table::{GBK_DECODED, GBK_ENCODED};
use super::string_methods::BytesProblem;

fn opens_a_pair(byte: u8) -> bool {
    (0x81..=0xfe).contains(&byte)
}

fn ends_a_pair(byte: u8) -> bool {
    (0x40..=0x7e).contains(&byte) || (0x80..=0xfe).contains(&byte)
}

/// Each character of a run of GBK bytes. A byte that opens a pair is read
/// together with the byte after it, when there is one.
pub(crate) fn gbk_characters(bytes: &[u8]) -> Vec<Vec<u8>> {
    let mut characters = Vec::new();
    let mut at = 0usize;
    while at < bytes.len() {
        let width = if opens_a_pair(bytes[at]) && at + 1 < bytes.len() {
            2
        } else {
            1
        };
        characters.push(bytes[at..at + width].to_vec());
        at += width;
    }
    characters
}

/// Where a run of GBK bytes stops spelling characters.
pub(crate) fn gbk_problem(bytes: &[u8]) -> Option<BytesProblem> {
    let mut at = 0usize;
    while at < bytes.len() {
        let byte = bytes[at];
        if byte <= 0x80 {
            at += 1;
            continue;
        }
        if !opens_a_pair(byte) {
            return Some((vec![byte], None, false));
        }
        match bytes.get(at + 1) {
            None => return Some((vec![byte], None, true)),
            Some(&trail) if ends_a_pair(trail) => at += 2,
            Some(&trail) => return Some((vec![byte], Some(trail), false)),
        }
    }
    None
}

/// Whether a run of bytes spells characters in GBK.
pub(crate) fn gbk_reads(bytes: &[u8]) -> bool {
    gbk_problem(bytes).is_none()
}

/// The character a run of GBK bytes opens with, alongside how many bytes it
/// took. None where the run opens no character GBK names.
pub(crate) fn gbk_character(bytes: &[u8]) -> Option<(char, usize)> {
    let first = *bytes.first()?;
    if first < 0x80 {
        return Some((char::from(first), 1));
    }
    let (packed, width) = if opens_a_pair(first) {
        let second = *bytes.get(1)?;
        ((u32::from(first) << 8) | u32::from(second), 2)
    } else {
        (u32::from(first), 1)
    };
    let at = GBK_DECODED
        .binary_search_by_key(&packed, |(held, _)| *held)
        .ok()?;
    char::from_u32(GBK_DECODED[at].1).map(|held| (held, width))
}

/// The text a run of GBK bytes spells. A byte GBK does not open a character
/// with stands for itself.
pub(crate) fn gbk_text(bytes: &[u8]) -> String {
    let mut written = String::with_capacity(bytes.len());
    let mut at = 0usize;
    while at < bytes.len() {
        match gbk_character(&bytes[at..]) {
            Some((character, width)) => {
                written.push(character);
                at += width;
            }
            None => {
                written.push(char::from(bytes[at]));
                at += 1;
            }
        }
    }
    written
}

/// The bytes GBK spells a whole text with, or the first character it has
/// no bytes for.
pub(crate) fn gbk_bytes(text: &str) -> Result<Vec<u8>, char> {
    let mut bytes = Vec::with_capacity(text.len());
    for character in text.chars() {
        let point = character as u32;
        if point < 0x80 {
            bytes.push(point as u8);
            continue;
        }
        let at = GBK_ENCODED
            .binary_search_by_key(&point, |(held, _)| *held)
            .map_err(|_| character)?;
        let packed = GBK_ENCODED[at].1;
        if packed > 0xff {
            bytes.push((packed >> 8) as u8);
        }
        bytes.push((packed & 0xff) as u8);
    }
    Ok(bytes)
}
