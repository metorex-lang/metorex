// Writing elements out as bytes.

use super::*;
use crate::object::View;

impl VirtualMachine {
    /// Write a packed result into the String a `buffer:` keyword named. The
    /// writing starts at the end of what the buffer holds, or at the offset a
    /// leading `@` directive names, and the buffer is the answer.
    pub(crate) fn pack_into_buffer(
        &mut self,
        buffer: Object,
        format: &str,
        packed: &Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let View::String(held) = buffer.view() else {
            return Ok(buffer);
        };
        if held.is_frozen() {
            return Err(self.frozen_modification_error(&buffer, position));
        }
        let existing = held.bytes().to_vec();
        let written = match packed.view() {
            View::String(text) => text.bytes().to_vec(),
            _ => Vec::new(),
        };
        let (start, tail) = match leading_pack_offset(format) {
            Some(offset) => (offset, written.get(offset..).unwrap_or(&[]).to_vec()),
            None => (existing.len(), written),
        };
        let mut made = existing;
        made.resize(start, 0);
        made.extend_from_slice(&tail);
        held.replace_bytes(made);
        Ok(buffer)
    }
}

/// The absolute offset a format's leading `@` directive names, where it has
/// one. Any other format writes at the end of what the buffer holds.
pub(crate) fn leading_pack_offset(format: &str) -> Option<usize> {
    let digits = format.strip_prefix('@')?;
    let counted: String = digits.chars().take_while(char::is_ascii_digit).collect();
    counted.parse().ok()
}
