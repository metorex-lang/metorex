//! Two ways for a String to hold a binary buffer, run through the buffer
//! operations protoboeuf-encode makes.
//!
//! `CharString` is Metorex's layout now: a Rust `String` with one character
//! per byte, so bytes 0x80 to 0xFF take two bytes of UTF-8 each. As in
//! `src/vm/native_methods`, `bytesize` copies the byte spelling, which is
//! kept until the text changes, `getbyte` reads the kept spelling, and
//! `setbyte` spells out every byte, changes one, and writes the text back.
//!
//! `ByteString` holds the bytes themselves, with the encoding as a tag
//! beside them, as CRuby does.

pub trait Buffer {
    fn new() -> Self;
    fn push_byte(&mut self, byte: u8);
    fn bytesize(&mut self) -> usize;
    fn setbyte(&mut self, index: usize, byte: u8);
    fn getbyte(&mut self, index: usize) -> u8;
}

pub struct CharString {
    text: String,
    spelled: Option<Vec<u8>>,
}

fn spell(text: &str) -> Vec<u8> {
    text.chars().map(|character| character as u8).collect()
}

impl Buffer for CharString {
    fn new() -> Self {
        CharString {
            text: String::new(),
            spelled: None,
        }
    }

    fn push_byte(&mut self, byte: u8) {
        self.text.push(byte as char);
        self.spelled = None;
    }

    fn bytesize(&mut self) -> usize {
        // `binary_bytes` hands back a copy of the kept spelling.
        self.spelled
            .get_or_insert_with(|| spell(&self.text))
            .to_vec()
            .len()
    }

    fn setbyte(&mut self, index: usize, byte: u8) {
        let mut bytes = spell(&self.text);
        bytes[index] = byte;
        self.text = bytes.iter().map(|byte| *byte as char).collect();
        self.spelled = None;
    }

    fn getbyte(&mut self, index: usize) -> u8 {
        self.spelled.get_or_insert_with(|| spell(&self.text))[index]
    }
}

pub struct ByteString {
    bytes: Vec<u8>,
    // The encoding a String is tagged with, carried as CRuby carries it.
    _encoding: u8,
}

impl Buffer for ByteString {
    fn new() -> Self {
        ByteString {
            bytes: Vec::new(),
            _encoding: 0,
        }
    }

    fn push_byte(&mut self, byte: u8) {
        self.bytes.push(byte);
    }

    fn bytesize(&mut self) -> usize {
        self.bytes.len()
    }

    fn setbyte(&mut self, index: usize, byte: u8) {
        self.bytes[index] = byte;
    }

    fn getbyte(&mut self, index: usize) -> u8 {
        self.bytes[index]
    }
}

fn push_varint<B: Buffer>(buffer: &mut B, mut value: u64) {
    loop {
        let byte = (value & 0x7f) as u8;
        value >>= 7;
        if value == 0 {
            buffer.push_byte(byte);
            return;
        }
        buffer.push_byte(byte | 0x80);
    }
}

/// Encode `lots` messages the way protoboeuf-encode's generated code does:
/// each holds 40 submessages, and each submessage's length is written into
/// a placeholder byte after it is encoded, which reads `bytesize` twice and
/// calls `setbyte` once. Each submessage holds eight varint fields. Answers
/// the sum of every byte read back with `getbyte`, as the decoder reads
/// them, to check both layouts wrote the same thing.
pub fn encode<B: Buffer>(lots: u64) -> u64 {
    let mut checksum = 0;
    for lot in 0..lots {
        let mut buffer = B::new();
        for vehicle in 0..40u64 {
            buffer.push_byte(0x0a);
            let placeholder = buffer.bytesize();
            buffer.push_byte(42);
            for field in 0..8u64 {
                buffer.push_byte(((field + 1) << 3) as u8);
                push_varint(
                    &mut buffer,
                    (lot * 7919 + vehicle * 104_729 + field * 1_299_709) % 4_000_000_000,
                );
            }
            let length = buffer.bytesize() - placeholder - 1;
            buffer.setbyte(placeholder, length as u8);
        }
        for index in 0..buffer.bytesize() {
            checksum += buffer.getbyte(index) as u64;
        }
    }
    checksum
}
