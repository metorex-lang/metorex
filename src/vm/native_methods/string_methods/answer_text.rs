// An answer built one piece at a time, each piece carrying the
// encoding it was written in.

use super::*;

/// An answer built one piece at a time, each piece written in an encoding of
/// its own. An answer holding only ASCII so far takes on the encoding of the
/// first piece that holds more, and a later piece that cannot be read in that
/// encoding is a CompatibilityError.
pub(crate) struct AnswerText {
    pub(crate) built: String,
    pub(crate) encoding: String,
    pub(crate) holds_only_ascii: bool,
    /// The bytes the pieces spell, which is what the answer holds once a
    /// piece held as bytes has joined it.
    bytes: Vec<u8>,
    joined_bytes: bool,
}

impl AnswerText {
    /// An answer with nothing in it yet, written in the encoding the text it
    /// is built from was written in.
    pub(crate) fn new(encoding: &str) -> Self {
        Self {
            built: String::new(),
            encoding: encoding.to_string(),
            holds_only_ascii: true,
            bytes: Vec::new(),
            joined_bytes: false,
        }
    }

    /// The String the pieces make. Pieces held as bytes are joined as
    /// bytes, so two halves of a character read as that character.
    pub(crate) fn into_string(self, holds_bytes: bool) -> crate::object::StringValue {
        if !self.joined_bytes {
            let made = crate::object::StringValue::with_encoding(self.built, &self.encoding);
            if holds_bytes || self.encoding == "ASCII-8BIT" {
                made.mark_bytes();
            }
            return made;
        }
        let made = crate::object::StringValue::binary(self.bytes);
        made.retag(self.encoding);
        made
    }

    /// Add a piece, held as bytes or as text.
    pub(crate) fn add_held(
        &mut self,
        piece: &str,
        encoding: &str,
        holds_bytes: bool,
        position: Position,
    ) -> Result<(), MetorexError> {
        if holds_bytes {
            self.add_bytes(piece, encoding, position)
        } else {
            self.add(piece, encoding, position)
        }
    }

    /// Add a piece held as bytes, as `add` does a piece of text.
    fn add_bytes(
        &mut self,
        piece: &str,
        encoding: &str,
        position: Position,
    ) -> Result<(), MetorexError> {
        let held = self.bytes.len();
        self.add(piece, encoding, position)?;
        if self.bytes.len() > held {
            self.joined_bytes = true;
            self.bytes.truncate(held);
            self.bytes
                .extend(crate::object::string_value::bytes_of_characters(piece));
        }
        Ok(())
    }

    /// What has been built so far.
    pub(crate) fn text(self) -> String {
        self.built
    }

    /// The encoding the pieces settled on.
    pub(crate) fn encoding(&self) -> String {
        self.encoding.clone()
    }

    pub(crate) fn add(
        &mut self,
        piece: &str,
        encoding: &str,
        position: Position,
    ) -> Result<(), MetorexError> {
        if piece.is_empty() {
            return Ok(());
        }
        // Text in an encoding that spells ASCII over several bytes never
        // joins text in another, however little either of them holds.
        if self.encoding != encoding
            && (!crate::vm::native_methods::class_methods::encoding_reads_alongside_ascii(encoding)
                || !crate::vm::native_methods::class_methods::encoding_reads_alongside_ascii(
                    &self.encoding,
                ))
        {
            return Err(crate::vm::errors::simple_exception(
                "Encoding::CompatibilityError",
                &crate::vm::native_methods::string_methods::incompatible_encodings_message(
                    &self.encoding,
                    encoding,
                ),
                position,
            ));
        }
        if !piece.is_ascii() {
            if self.holds_only_ascii {
                self.encoding = encoding.to_string();
                self.holds_only_ascii = false;
            } else if self.encoding != encoding {
                return Err(crate::vm::errors::simple_exception(
                    "Encoding::CompatibilityError",
                    &crate::vm::native_methods::string_methods::incompatible_encodings_message(
                        &self.encoding,
                        encoding,
                    ),
                    position,
                ));
            }
        } else if self.holds_only_ascii
            && !crate::vm::native_methods::class_methods::encoding_reads_alongside_ascii(
                &self.encoding,
            )
        {
            self.encoding = encoding.to_string();
        }
        self.built.push_str(piece);
        self.bytes.extend_from_slice(piece.as_bytes());
        Ok(())
    }
}
