//! YAML read into the events Psych's parser hands its handler, and events
//! written back out as YAML, in the shapes libyaml gives them so MRI's own
//! Psych library runs on top unchanged.

mod emitter;
mod parser;
mod scanner;

pub use emitter::{Emitter, EmitterOptions};
pub use parser::parse;

/// A place in the text: the character it is at, counted from the start, and
/// its line and column, each counted from zero.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Mark {
    pub index: usize,
    pub line: usize,
    pub column: usize,
}

/// How a scalar is written, numbered as libyaml and Psych number them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ScalarStyle {
    Any = 0,
    Plain = 1,
    SingleQuoted = 2,
    DoubleQuoted = 3,
    Literal = 4,
    Folded = 5,
}

impl ScalarStyle {
    pub fn from_number(number: i64) -> ScalarStyle {
        match number {
            1 => ScalarStyle::Plain,
            2 => ScalarStyle::SingleQuoted,
            3 => ScalarStyle::DoubleQuoted,
            4 => ScalarStyle::Literal,
            5 => ScalarStyle::Folded,
            _ => ScalarStyle::Any,
        }
    }
}

/// How a sequence or mapping is written: 1 in block form and 2 in flow
/// form, 0 leaving it to the emitter.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CollectionStyle {
    Any = 0,
    Block = 1,
    Flow = 2,
}

impl CollectionStyle {
    pub fn from_number(number: i64) -> CollectionStyle {
        match number {
            1 => CollectionStyle::Block,
            2 => CollectionStyle::Flow,
            _ => CollectionStyle::Any,
        }
    }
}

/// Why text could not be read, with the fields Psych's SyntaxError takes.
#[derive(Clone, Debug)]
pub struct YamlError {
    pub context: Option<String>,
    pub context_mark: Mark,
    pub problem: String,
    pub problem_mark: Mark,
    pub offset: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub enum EventKind {
    StreamStart,
    StreamEnd,
    DocumentStart {
        version: Option<(u32, u32)>,
        tags: Vec<(String, String)>,
        implicit: bool,
    },
    DocumentEnd {
        implicit: bool,
    },
    Alias {
        anchor: String,
    },
    Scalar {
        anchor: Option<String>,
        tag: Option<String>,
        value: String,
        plain_implicit: bool,
        quoted_implicit: bool,
        style: ScalarStyle,
    },
    SequenceStart {
        anchor: Option<String>,
        tag: Option<String>,
        implicit: bool,
        style: CollectionStyle,
    },
    SequenceEnd,
    MappingStart {
        anchor: Option<String>,
        tag: Option<String>,
        implicit: bool,
        style: CollectionStyle,
    },
    MappingEnd,
}

#[derive(Clone, Debug)]
pub struct Event {
    pub kind: EventKind,
    pub start: Mark,
    pub end: Mark,
    /// Where the scanner stood once the event was read, which is what
    /// `Psych::Parser#mark` answers while the handler runs.
    pub reached: Mark,
}

/// The characters of the text, with the reader's complaint about the first
/// place it stops being text libyaml reads: an invalid UTF-8 sequence, or a
/// character YAML does not allow.
fn decode(bytes: &[u8]) -> (Vec<char>, Option<scanner::ReaderTrouble>) {
    let mut characters = Vec::with_capacity(bytes.len());
    let mut offset = 0;
    // A byte order mark opens the stream and is read as no character.
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        offset = 3;
    }
    while offset < bytes.len() {
        let rest = &bytes[offset..];
        let (spelled, width) = match std::str::from_utf8(&rest[..rest.len().min(4)]) {
            Ok(text) => {
                let held = text.chars().next().unwrap_or('\0');
                (held, held.len_utf8())
            }
            Err(trouble) if trouble.valid_up_to() > 0 => {
                let text = std::str::from_utf8(&rest[..trouble.valid_up_to()]).unwrap_or("");
                let held = text.chars().next().unwrap_or('\0');
                (held, held.len_utf8())
            }
            Err(trouble) => {
                let lead = rest[0];
                let problem = if trouble.error_len().is_none() {
                    "incomplete UTF-8 octet sequence"
                } else if lead & 0xC0 == 0x80 || lead >= 0xF8 {
                    "invalid leading UTF-8 octet"
                } else {
                    "invalid trailing UTF-8 octet"
                };
                let trouble = scanner::ReaderTrouble {
                    at: characters.len(),
                    offset,
                    problem,
                };
                return (characters, Some(trouble));
            }
        };
        if !allowed_character(spelled) {
            let trouble = scanner::ReaderTrouble {
                at: characters.len(),
                offset,
                problem: "control characters are not allowed",
            };
            return (characters, Some(trouble));
        }
        characters.push(spelled);
        offset += width;
    }
    (characters, None)
}

/// The characters libyaml's reader lets into a stream.
fn allowed_character(held: char) -> bool {
    matches!(held,
        '\t' | '\n' | '\r' | '\u{20}'..='\u{7e}' | '\u{85}' | '\u{a0}'..='\u{d7ff}'
            | '\u{e000}'..='\u{fffd}' | '\u{10000}'..='\u{10ffff}')
}
