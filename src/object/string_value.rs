//! The bytes a Ruby String holds, with the encoding they are tagged with.
//!
//! A string reads its bytes as text one of two ways. Most read them as UTF-8.
//! One tagged binary or with an encoding that spells a character in more than
//! one byte, and one marked as holding bytes, reads each byte as the character
//! with that code point, so its text is never anything but a spelling of its
//! bytes.

use std::cell::{Cell, Ref, RefCell};
use std::rc::Rc;

/// The encoding a string literal carries.
pub const DEFAULT_ENCODING: &str = "UTF-8";

/// Whether a string tagged `encoding` reads each byte as a character of its
/// own, whatever else it says of itself.
fn encoding_reads_bytes(encoding: &str) -> bool {
    matches!(
        encoding,
        "ASCII-8BIT"
            | "BINARY"
            | "UTF-16"
            | "UTF-16BE"
            | "UTF-16LE"
            | "UTF-32"
            | "UTF-32BE"
            | "UTF-32LE"
    )
}

/// The bytes text spells, one per character below 256 and the UTF-8 form of
/// every character above.
pub fn bytes_of_characters(text: &str) -> Vec<u8> {
    if text.is_ascii() {
        return text.as_bytes().to_vec();
    }
    let mut bytes = Vec::with_capacity(text.len());
    for character in text.chars() {
        if (character as u32) < 256 {
            bytes.push(character as u8);
        } else {
            let mut buffer = [0u8; 4];
            bytes.extend_from_slice(character.encode_utf8(&mut buffer).as_bytes());
        }
    }
    bytes
}

/// The one copy of an encoding's name every string tagged with it shares.
/// The names are the encodings Ruby knows and the few a program adds, so
/// each is kept for the life of the process.
fn shared_encoding_name(name: String) -> &'static str {
    thread_local! {
        static NAMES: RefCell<std::collections::HashSet<&'static str>> =
            RefCell::new([DEFAULT_ENCODING, "ASCII-8BIT", "US-ASCII"].into_iter().collect());
    }
    NAMES.with(|names| {
        if let Some(shared) = names.borrow().get(name.as_str()) {
            return *shared;
        }
        let shared: &'static str = String::leak(name);
        names.borrow_mut().insert(shared);
        shared
    })
}

/// The text of bytes read one character per byte.
fn characters_of_bytes(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| *byte as char).collect()
}

#[derive(Debug)]
pub struct StringValue {
    /// What the string holds. Ruby lets a string change what it holds while
    /// every reference to it sees the change, so the bytes sit behind a cell
    /// rather than being fixed when the string is made.
    bytes: RefCell<Vec<u8>>,
    encoding: Cell<&'static str>,
    /// The id this string answers, handed out the first time one is asked
    /// for. A counter rather than an address, since an address a dropped
    /// string leaves behind can be handed to the next one.
    object_id: Cell<u64>,
    /// Whether the string refuses to change. `freeze` sets it, and every
    /// method that would change the bytes checks it first.
    frozen: Cell<bool>,
    /// Whether the bytes are read one character per byte under a text
    /// encoding. A run of bytes read off a file or unpacked from a format is
    /// built this way, and keeps saying so after it is tagged with a text
    /// encoding.
    holds_bytes: Cell<bool>,
    /// Whether something else is reading the string right now and refuses to
    /// have it changed underneath. An IO::Buffer over a string sets this for
    /// as long as it is in use.
    borrowed: Cell<bool>,
    /// Whether the string is one of the interned copies `String#-@` hands
    /// back. Ruby refuses a singleton class on one of those, since the same
    /// string stands for every use of that text.
    deduplicated: Cell<bool>,
    /// What a few strings carry beside their text, made the first time one
    /// is given any of it.
    notes: RefCell<Option<Box<Notes>>>,
    /// Whether the bytes are all ASCII, worked out the first time it is
    /// asked and forgotten whenever the bytes change.
    ascii: Cell<Option<bool>>,
    /// What the text was read as, made the first time a reading is asked
    /// for and emptied whenever the text changes.
    readings: RefCell<Option<Box<Readings>>>,
}

#[derive(Debug, Default, Clone)]
struct Notes {
    /// The warning a change to this string prints, for one Ruby hands back
    /// with notice that it will be frozen in a later release. Cleared once
    /// the warning has been given, so it is printed only the first time.
    chilled: Option<String>,
    /// The pointer a `pack` with 'P' or 'p' left on the string it built, which
    /// names the string those directives wrote the address of. Zero where the
    /// string carries none, which is every string but a packed one.
    pointer: u64,
    /// Where the literal that made this string was written, as `file:line`.
    /// Only `--debug-frozen-string-literal` records it, and everything else
    /// leaves it empty.
    created_at: Option<String>,
}

#[derive(Debug, Default)]
struct Readings {
    /// The text of a string read one character per byte that holds a byte
    /// above 0x7F, built the first time it is asked for and forgotten
    /// whenever the bytes or the way they are read change.
    spelled: Option<String>,
    /// Where every `CHARACTERS_PER_MARK`th character starts, built the first
    /// time a character is looked up by position and forgotten whenever the
    /// text changes.
    character_marks: Option<Rc<CharacterMarks>>,
    /// The text's Rust characters, which a pattern is matched over, built
    /// the first time a match asks and forgotten whenever the text changes.
    letters: Option<Rc<[char]>>,
}

/// How many characters apart the byte offsets in `CharacterMarks` are.
const CHARACTERS_PER_MARK: usize = 64;

/// The byte offset of every `CHARACTERS_PER_MARK`th character of a text, and
/// how many characters it has.
#[derive(Debug)]
struct CharacterMarks {
    offsets: Vec<usize>,
    count: usize,
}

impl CharacterMarks {
    fn of(text: &str) -> Self {
        let mut offsets = Vec::with_capacity(text.len() / CHARACTERS_PER_MARK + 1);
        let mut count = 0;
        for (offset, _) in text.char_indices() {
            if count % CHARACTERS_PER_MARK == 0 {
                offsets.push(offset);
            }
            count += 1;
        }
        Self { offsets, count }
    }

    /// The byte offset character `index` starts at, or the text's length for
    /// the index just past its last character.
    fn byte_offset(&self, text: &str, index: usize) -> usize {
        if index >= self.count {
            return text.len();
        }
        let start = self.offsets[index / CHARACTERS_PER_MARK];
        let skipped = index % CHARACTERS_PER_MARK;
        text[start..]
            .char_indices()
            .nth(skipped)
            .map_or(text.len(), |(offset, _)| start + offset)
    }
}

impl StringValue {
    /// The heap bytes the string's text holds, for the memory report.
    pub(crate) fn held_bytes(&self) -> usize {
        self.bytes.try_borrow().map_or(0, |bytes| bytes.capacity())
    }
}

impl StringValue {
    fn built(bytes: Vec<u8>, encoding: String, holds_bytes: bool) -> Self {
        Self {
            bytes: RefCell::new(bytes),
            encoding: Cell::new(shared_encoding_name(encoding)),
            object_id: Cell::new(0),
            frozen: Cell::new(false),
            holds_bytes: Cell::new(holds_bytes),
            borrowed: Cell::new(false),
            deduplicated: Cell::new(false),
            notes: RefCell::new(None),
            ascii: Cell::new(None),
            readings: RefCell::new(None),
        }
    }

    fn note(&self, change: impl FnOnce(&mut Notes)) {
        change(self.notes.borrow_mut().get_or_insert_with(Box::default));
    }

    fn noted<T>(&self, read: impl FnOnce(&Notes) -> T) -> Option<T> {
        self.notes.borrow().as_deref().map(read)
    }

    fn reading<T: Clone>(&self, read: impl FnOnce(&Readings) -> Option<&T>) -> Option<T> {
        self.readings.borrow().as_deref().and_then(read).cloned()
    }

    fn keep_reading(&self, keep: impl FnOnce(&mut Readings)) {
        keep(self.readings.borrow_mut().get_or_insert_with(Box::default));
    }

    /// Text tagged with the encoding a literal carries.
    pub fn new(text: impl Into<String>) -> Self {
        Self::built(
            text.into().into_bytes(),
            DEFAULT_ENCODING.to_string(),
            false,
        )
    }

    /// Text tagged with an encoding named outright. Under an encoding that
    /// reads each byte as a character, the text spells the bytes.
    pub fn with_encoding(text: impl Into<String>, encoding: impl Into<String>) -> Self {
        let encoding = encoding.into();
        let text = text.into();
        let bytes = if encoding_reads_bytes(&encoding) {
            bytes_of_characters(&text)
        } else {
            text.into_bytes()
        };
        Self::built(bytes, encoding, false)
    }

    /// Text whose characters stand for the bytes they were read from.
    pub fn from_bytes(text: impl Into<String>) -> Self {
        Self::built(
            bytes_of_characters(&text.into()),
            "ASCII-8BIT".to_string(),
            true,
        )
    }

    /// Bytes tagged binary.
    pub fn binary(bytes: Vec<u8>) -> Self {
        Self::built(bytes, "ASCII-8BIT".to_string(), true)
    }

    /// Bytes tagged with the named encoding. Bytes that are not UTF-8 are
    /// read one character per byte.
    pub fn of_bytes(bytes: Vec<u8>, encoding: impl Into<String>) -> Self {
        let holds_bytes = std::str::from_utf8(&bytes).is_err();
        Self::built(bytes, encoding.into(), holds_bytes)
    }

    /// Whether each byte reads as a character of its own.
    pub fn reads_bytes(&self) -> bool {
        self.holds_bytes.get() || encoding_reads_bytes(self.encoding.get())
    }

    /// Whether the string is marked as holding bytes under a text encoding.
    pub fn holds_bytes(&self) -> bool {
        self.holds_bytes.get()
    }

    /// Change how the bytes are read, keeping the text they read as.
    fn reread(&self, change: impl FnOnce()) {
        let before = self.reads_bytes();
        change();
        let after = self.reads_bytes();
        if before == after {
            return;
        }
        let respelled = {
            let bytes = self.bytes.borrow();
            if after {
                bytes_of_characters(std::str::from_utf8(&bytes).unwrap_or_default())
            } else {
                characters_of_bytes(&bytes).into_bytes()
            }
        };
        *self.bytes.borrow_mut() = respelled;
        self.forget_readings();
    }

    /// Say that the characters stand for bytes, which is what a copy built
    /// from one that does has to carry.
    pub fn mark_bytes(&self) {
        self.reread(|| self.holds_bytes.set(true));
    }

    /// Say that the characters stand for text again, which is what reading a
    /// run of bytes back through an encoding that spells them leaves.
    pub fn clear_bytes(&self) {
        self.reread(|| self.holds_bytes.set(false));
    }

    /// The name of the encoding this string says it is in.
    pub fn encoding_name(&self) -> String {
        self.encoding.get().to_string()
    }

    /// Tag the string as being in another encoding, keeping the text it
    /// reads as. Every reference to the same string sees the new tag.
    pub fn set_encoding(&self, encoding: impl Into<String>) {
        let encoding = shared_encoding_name(encoding.into());
        self.reread(|| self.encoding.set(encoding));
    }

    /// Tag the string as being in another encoding, keeping its bytes, which
    /// is what Ruby's `force_encoding` does. Bytes read one per byte keep
    /// being read that way unless the new encoding is UTF-8 or US-ASCII and
    /// they spell it.
    pub fn retag(&self, encoding: impl Into<String>) {
        let encoding = encoding.into();
        let broken = std::str::from_utf8(&self.bytes.borrow()).is_err();
        let holds_bytes = if matches!(encoding.as_str(), "UTF-8" | "US-ASCII") {
            broken
        } else {
            broken || self.reads_bytes() || encoding_reads_bytes(&encoding)
        };
        self.encoding.set(shared_encoding_name(encoding));
        self.holds_bytes.set(holds_bytes);
        self.forget_readings();
    }

    /// The bytes the string holds.
    pub fn bytes(&self) -> Ref<'_, [u8]> {
        Ref::map(self.bytes.borrow(), |held| held.as_slice())
    }

    /// How many bytes the string holds.
    pub fn byte_len(&self) -> usize {
        self.bytes.borrow().len()
    }

    /// Replace the bytes the string holds.
    pub fn replace_bytes(&self, bytes: Vec<u8>) {
        *self.bytes.borrow_mut() = bytes;
        self.settle();
    }

    /// Replace the byte at `index`, which is within the string.
    pub fn set_byte(&self, index: usize, byte: u8) {
        self.bytes.borrow_mut()[index] = byte;
        self.settle();
    }

    /// Add bytes to the end of the string.
    pub fn append_bytes(&self, extra: &[u8]) {
        self.bytes.borrow_mut().extend_from_slice(extra);
        self.settle();
    }

    /// After a change to the bytes, read them one per byte when they no
    /// longer spell UTF-8.
    fn settle(&self) {
        self.forget_readings();
        if !self.reads_bytes() && std::str::from_utf8(&self.bytes.borrow()).is_err() {
            self.holds_bytes.set(true);
        }
    }

    /// The text the bytes read as. The borrow lasts as long as the value is
    /// held, so a caller that changes the string has to let it go first.
    pub fn as_str(&self) -> Ref<'_, str> {
        if !self.reads_bytes() || self.is_ascii() {
            // SAFETY: bytes read as UTF-8 are kept UTF-8 by `settle` and by
            // every constructor, and ASCII bytes are UTF-8.
            return Ref::map(self.bytes.borrow(), |held| unsafe {
                std::str::from_utf8_unchecked(held)
            });
        }
        let spelled = self
            .readings
            .borrow()
            .as_deref()
            .is_some_and(|readings| readings.spelled.is_some());
        if !spelled {
            let text = characters_of_bytes(&self.bytes.borrow());
            self.keep_reading(|readings| readings.spelled = Some(text));
        }
        Ref::map(self.readings.borrow(), |held| {
            held.as_deref()
                .and_then(|readings| readings.spelled.as_deref())
                .unwrap_or_default()
        })
    }

    /// The bytes text spells under the way this string reads its bytes.
    fn spell(&self, text: &str) -> Vec<u8> {
        if self.reads_bytes() {
            bytes_of_characters(text)
        } else {
            text.as_bytes().to_vec()
        }
    }

    /// Whether every byte is ASCII.
    pub fn is_ascii(&self) -> bool {
        if let Some(known) = self.ascii.get() {
            return known;
        }
        let found = self.bytes.borrow().is_ascii();
        self.ascii.set(Some(found));
        found
    }

    fn forget_readings(&self) {
        self.ascii.set(None);
        if let Some(readings) = self.readings.borrow_mut().as_deref_mut() {
            *readings = Readings::default();
        }
    }

    /// The text's Rust characters, in order.
    pub fn letters(&self) -> Rc<[char]> {
        if let Some(letters) = self.reading(|readings| readings.letters.as_ref()) {
            return letters;
        }
        let letters: Rc<[char]> = self.as_str().chars().collect();
        self.keep_reading(|readings| readings.letters = Some(Rc::clone(&letters)));
        letters
    }

    fn character_marks(&self) -> Rc<CharacterMarks> {
        if let Some(marks) = self.reading(|readings| readings.character_marks.as_ref()) {
            return marks;
        }
        let marks = Rc::new(CharacterMarks::of(&self.as_str()));
        self.keep_reading(|readings| readings.character_marks = Some(Rc::clone(&marks)));
        marks
    }

    /// How many Rust characters the text holds.
    pub fn character_count(&self) -> usize {
        if self.is_ascii() {
            return self.byte_len();
        }
        if self.reads_bytes() {
            return self.byte_len();
        }
        self.character_marks().count
    }

    /// The text of characters `from` up to but not including `to`, counted
    /// in Rust characters. Both are clamped to the text's length.
    pub fn character_slice(&self, from: usize, to: usize) -> String {
        if self.is_ascii() || self.reads_bytes() {
            let bytes = self.bytes.borrow();
            let end = to.min(bytes.len());
            let cut = &bytes[from.min(end)..end];
            if let Ok(text) = std::str::from_utf8(cut)
                && text.is_ascii()
            {
                return text.to_string();
            }
            return characters_of_bytes(cut);
        }
        let marks = self.character_marks();
        let text = self.as_str();
        let start = marks.byte_offset(&text, from);
        let end = marks.byte_offset(&text, to.max(from));
        text[start..end].to_string()
    }

    /// A copy of the text, for a caller that keeps it past a change.
    pub fn to_text(&self) -> String {
        self.as_str().to_string()
    }

    /// Replace what the string holds. Every reference to the same string
    /// sees the new text, which is what Ruby's in-place methods do.
    pub fn replace_text(&self, text: impl Into<String>) {
        let bytes = self.spell(&text.into());
        *self.bytes.borrow_mut() = bytes;
        self.forget_readings();
    }

    /// Add text to the end of what the string holds.
    pub fn append_text(&self, extra: &str) {
        let ascii = self.ascii.get();
        let bytes = self.spell(extra);
        self.bytes.borrow_mut().extend_from_slice(&bytes);
        self.forget_readings();
        if ascii == Some(true) && extra.is_ascii() {
            self.ascii.set(Some(true));
        }
    }

    /// Change the text through a function of what it holds now.
    pub fn update_text(&self, change: impl FnOnce(&str) -> String) {
        let made = change(&self.as_str());
        self.replace_text(made);
    }

    /// Whether something else is reading the string and refuses a change.
    pub fn is_borrowed(&self) -> bool {
        self.borrowed.get()
    }

    /// Say that something else is reading the string, or that it is done.
    pub fn set_borrowed(&self, borrowed: bool) {
        self.borrowed.set(borrowed);
    }

    /// Give the string notice that it will be frozen in a later release, so
    /// the first change made to it says so.
    pub fn chill(&self, warning: String) {
        self.note(|notes| notes.chilled = Some(warning));
    }

    /// The pointer `pack` left on this string, or zero where it left none.
    pub fn pointer(&self) -> u64 {
        self.noted(|notes| notes.pointer).unwrap_or(0)
    }

    /// Mark this string as naming the run of text a pointer stands for.
    pub fn set_pointer(&self, named: u64) {
        self.note(|notes| notes.pointer = named);
    }

    /// Whether the string carries notice that it will be frozen in a later
    /// release.
    pub fn is_chilled(&self) -> bool {
        self.noted(|notes| notes.chilled.is_some()).unwrap_or(false)
    }

    /// The notice a change to this string prints, taken so it prints once.
    pub fn take_chill(&self) -> Option<String> {
        self.notes
            .borrow_mut()
            .as_deref_mut()
            .and_then(|notes| notes.chilled.take())
    }

    /// Record where the literal that made this string was written.
    pub fn set_created_at(&self, written_at: String) {
        self.note(|notes| notes.created_at = Some(written_at));
    }

    /// Where the literal that made this string was written, for a run that
    /// asked to be told.
    pub fn created_at(&self) -> Option<String> {
        self.noted(|notes| notes.created_at.clone()).flatten()
    }

    /// Whether the string is one of the interned copies `String#-@` answers.
    pub fn is_deduplicated(&self) -> bool {
        self.deduplicated.get()
    }

    /// Say the string is one of the interned copies.
    pub fn mark_deduplicated(&self) {
        self.deduplicated.set(true);
        self.frozen.set(true);
    }

    /// Whether the string refuses to change.
    pub fn is_frozen(&self) -> bool {
        self.frozen.get()
    }

    /// Refuse every change from here on. Every reference to the same string
    /// sees it, which is what Ruby's `freeze` does.
    pub fn freeze(&self) {
        self.frozen.set(true);
    }

    /// Let the string change again, which C does by clearing the FREEZE
    /// flag in its `RBasic`.
    pub fn thaw(&self) {
        self.frozen.set(false);
    }

    /// The id this string answers. The first ask takes the next one from the
    /// counter handed in, and every ask after answers the same.
    pub fn object_id(&self, next: impl FnOnce() -> u64) -> u64 {
        if self.object_id.get() == 0 {
            self.object_id.set(next());
        }
        self.object_id.get()
    }
}

impl Clone for StringValue {
    /// A copy holds the same bytes under an id of its own, since Ruby's `dup`
    /// answers a string that changes on its own.
    fn clone(&self) -> Self {
        let copy = Self::built(
            self.bytes.borrow().clone(),
            self.encoding.get().to_string(),
            self.holds_bytes.get(),
        );
        if let Some(notes) = self.notes.borrow().as_deref() {
            copy.note(|copied| {
                copied.pointer = notes.pointer;
                copied.created_at = notes.created_at.clone();
            });
        }
        copy.ascii.set(self.ascii.get());
        if let Some(readings) = self.readings.borrow().as_deref() {
            copy.keep_reading(|copied| {
                copied.character_marks = readings.character_marks.clone();
                copied.letters = readings.letters.clone();
            });
        }
        copy
    }
}

impl PartialEq for StringValue {
    /// Two strings are equal when they read as the same text. Ruby compares
    /// an ASCII-only string across encodings the same way.
    fn eq(&self, other: &Self) -> bool {
        *self.as_str() == *other.as_str()
    }
}

impl Eq for StringValue {}

/// Strings order by their text, which is what `<` and `<=>` compare.
impl PartialOrd for StringValue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for StringValue {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_str().cmp(&other.as_str())
    }
}

/// Comparing against plain text reads the text this holds.
impl PartialEq<String> for StringValue {
    fn eq(&self, other: &String) -> bool {
        *self.as_str() == **other
    }
}

impl PartialEq<StringValue> for String {
    fn eq(&self, other: &StringValue) -> bool {
        **self == *other.as_str()
    }
}

impl PartialEq<str> for StringValue {
    fn eq(&self, other: &str) -> bool {
        *self.as_str() == *other
    }
}

impl PartialEq<&str> for StringValue {
    fn eq(&self, other: &&str) -> bool {
        *self.as_str() == **other
    }
}

impl std::hash::Hash for StringValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

impl std::fmt::Display for StringValue {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", &*self.as_str())
    }
}

/// Reading a StringValue out as plain text, which is what everything that
/// wants a `String` from one does.
impl From<StringValue> for String {
    fn from(held: StringValue) -> Self {
        held.to_text()
    }
}

impl From<String> for StringValue {
    fn from(text: String) -> Self {
        Self::new(text)
    }
}

impl From<&str> for StringValue {
    fn from(text: &str) -> Self {
        Self::new(text)
    }
}
