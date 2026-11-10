//! The Onigmo functions `ruby/oniguruma.h` reaches through its encoding
//! macros: a Unicode property's number from its name, and its code point
//! ranges from that number, read from the tables metorex's own patterns
//! match against.

use super::encodings::RbEncoding;
use crate::regexp::{numbered_property_ranges, property_number};
use std::sync::OnceLock;

/// What Onigmo answers for a property name it does not know.
const INVALID_CHAR_PROPERTY_NAME: i32 = -223;
/// What Onigmo answers for a property number past its last.
const TYPE_BUG: i32 = -6;
/// The first code point a UTF-8 character spells with more than one byte.
const SINGLE_BYTE_LIMIT: u32 = 0x80;

/// Each property's ranges as Onigmo lays them out for C: their count, then
/// the first and last code point of each.
fn onigmo_range_lists() -> &'static [Box<[u32]>] {
    static LISTS: OnceLock<Vec<Box<[u32]>>> = OnceLock::new();
    LISTS.get_or_init(|| {
        (0..)
            .map_while(numbered_property_ranges)
            .map(|ranges| {
                std::iter::once(ranges.len() as u32)
                    .chain(ranges.iter().flat_map(|(first, last)| [*first, *last]))
                    .collect()
            })
            .collect()
    })
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_metorex_onigenc_property_name_to_ctype(
    _encoding: *const RbEncoding,
    start: *const u8,
    end: *const u8,
) -> i32 {
    // SAFETY: the macro's callers pass the start and end of one name.
    let name = unsafe { std::slice::from_raw_parts(start, end.offset_from(start) as usize) };
    if !name.is_ascii() {
        return INVALID_CHAR_PROPERTY_NAME;
    }
    let plain: String = name
        .iter()
        .filter(|byte| !matches!(byte, b' ' | b'-' | b'_'))
        .map(|byte| byte.to_ascii_lowercase() as char)
        .collect();
    property_number(&plain).map_or(INVALID_CHAR_PROPERTY_NAME, |number| number as i32)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_metorex_onigenc_get_ctype_code_range(
    _encoding: *const RbEncoding,
    ctype: u32,
    single_byte_out: *mut u32,
    ranges: *mut *const u32,
) -> i32 {
    let Some(list) = onigmo_range_lists().get(ctype as usize) else {
        return TYPE_BUG;
    };
    // SAFETY: the macro's callers pass places to write each answer.
    unsafe {
        *single_byte_out = SINGLE_BYTE_LIMIT;
        *ranges = list.as_ptr();
    }
    0
}
