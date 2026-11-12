// The compressed stream formats zlib reads and writes: the two checksums it
// keeps, the DEFLATE encoding underneath, and the wrappers that carry it.

/// The table CRC-32 is computed from, one entry per byte value.
fn crc_table() -> [u32; 256] {
    let mut table = [0u32; 256];
    for (index, slot) in table.iter_mut().enumerate() {
        let mut held = index as u32;
        for _ in 0..8 {
            held = if held & 1 == 1 {
                0xedb8_8320 ^ (held >> 1)
            } else {
                held >> 1
            };
        }
        *slot = held;
    }
    table
}

/// The CRC-32 of `bytes`, continuing from `running`.
pub(crate) fn crc32(running: u32, bytes: &[u8]) -> u32 {
    let table = crc_table();
    let mut held = !running;
    for byte in bytes {
        held = table[((held ^ u32::from(*byte)) & 0xff) as usize] ^ (held >> 8);
    }
    !held
}

/// The whole CRC-32 table, which `Zlib.crc_table` hands out.
pub(crate) fn crc_table_values() -> Vec<u32> {
    crc_table().to_vec()
}

/// The Adler-32 of `bytes`, continuing from `running`.
pub(crate) fn adler32(running: u32, bytes: &[u8]) -> u32 {
    const MODULUS: u32 = 65521;
    let mut low = running & 0xffff;
    let mut high = (running >> 16) & 0xffff;
    for byte in bytes {
        low = (low + u32::from(*byte)) % MODULUS;
        high = (high + low) % MODULUS;
    }
    (high << 16) | low
}

/// Reads a DEFLATE stream one bit at a time, least significant bit first.
struct BitReader<'a> {
    bytes: &'a [u8],
    at: usize,
    bit: u32,
}

impl<'a> BitReader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            at: 0,
            bit: 0,
        }
    }

    fn read_bit(&mut self) -> Option<u32> {
        let byte = *self.bytes.get(self.at)?;
        let held = (u32::from(byte) >> self.bit) & 1;
        self.bit += 1;
        if self.bit == 8 {
            self.bit = 0;
            self.at += 1;
        }
        Some(held)
    }

    fn read_bits(&mut self, count: u32) -> Option<u32> {
        let mut held = 0u32;
        for place in 0..count {
            held |= self.read_bit()? << place;
        }
        Some(held)
    }

    /// Move to the start of the next byte, which a stored block begins on.
    fn align(&mut self) {
        if self.bit != 0 {
            self.bit = 0;
            self.at += 1;
        }
    }
}

/// A canonical Huffman code, held as the counts and symbols the decoder walks.
struct Huffman {
    counts: Vec<u16>,
    symbols: Vec<u16>,
}

impl Huffman {
    /// Build the code the given lengths describe, which is how DEFLATE names
    /// both of its alphabets.
    fn from_lengths(lengths: &[u8]) -> Self {
        const MAX_BITS: usize = 15;
        let mut counts = vec![0u16; MAX_BITS + 1];
        for length in lengths {
            counts[*length as usize] += 1;
        }
        counts[0] = 0;
        let mut offsets = [0u16; MAX_BITS + 2];
        for bits in 1..=MAX_BITS {
            offsets[bits + 1] = offsets[bits] + counts[bits];
        }
        let mut symbols = vec![0u16; lengths.len()];
        for (symbol, length) in lengths.iter().enumerate() {
            if *length != 0 {
                symbols[offsets[*length as usize] as usize] = symbol as u16;
                offsets[*length as usize] += 1;
            }
        }
        Self { counts, symbols }
    }

    /// Whether the lengths make a code zlib accepts: none over-subscribed,
    /// and none incomplete except where `lone_code_allowed` lets a single
    /// code of one bit stand alone, as a literal or distance code may.
    fn lengths_accepted(lengths: &[u8], lone_code_allowed: bool) -> bool {
        const MAX_BITS: usize = 15;
        let mut counts = [0i32; MAX_BITS + 1];
        for length in lengths {
            counts[*length as usize] += 1;
        }
        let longest = (1..=MAX_BITS)
            .rev()
            .find(|bits| counts[*bits] != 0)
            .unwrap_or(0);
        if longest == 0 {
            return true;
        }
        let mut left = 1i32;
        for count in counts.iter().skip(1) {
            left <<= 1;
            left -= count;
            if left < 0 {
                return false;
            }
        }
        left == 0 || (lone_code_allowed && longest == 1)
    }

    fn decode(&self, reader: &mut BitReader<'_>) -> Option<u16> {
        let mut code = 0i32;
        let mut first = 0i32;
        let mut index = 0i32;
        for bits in 1..self.counts.len() {
            code |= reader.read_bit()? as i32;
            let count = i32::from(self.counts[bits]);
            if code - first < count {
                return self.symbols.get((index + (code - first)) as usize).copied();
            }
            index += count;
            first = (first + count) << 1;
            code <<= 1;
        }
        None
    }
}

/// How much each length symbol stands for, and how many extra bits follow it.
const LENGTH_BASE: [u16; 29] = [
    3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131,
    163, 195, 227, 258,
];
const LENGTH_EXTRA: [u32; 29] = [
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0,
];
const DISTANCE_BASE: [u16; 30] = [
    1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537,
    2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577,
];
const DISTANCE_EXTRA: [u32; 30] = [
    0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13,
    13,
];

/// The fixed code DEFLATE uses when a block names no code of its own.
fn fixed_codes() -> (Huffman, Huffman) {
    let mut lengths = vec![8u8; 288];
    lengths[144..256].fill(9);
    lengths[256..280].fill(7);
    let distances = vec![5u8; 30];
    (
        Huffman::from_lengths(&lengths),
        Huffman::from_lengths(&distances),
    )
}

/// Why a stream could not be read: it stopped short, or it is not a stream,
/// for the reason zlib gives.
enum Stop {
    Short,
    Invalid(&'static str),
}

/// A read that ran out of bytes stopped short.
fn bits(read: Option<u32>) -> Result<u32, Stop> {
    read.ok_or(Stop::Short)
}

/// The code lengths a dynamic block names for itself.
fn dynamic_codes(reader: &mut BitReader<'_>) -> Result<(Huffman, Huffman), Stop> {
    const ORDER: [usize; 19] = [
        16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
    ];
    let literal_count = bits(reader.read_bits(5))? as usize + 257;
    let distance_count = bits(reader.read_bits(5))? as usize + 1;
    let code_count = bits(reader.read_bits(4))? as usize + 4;
    if literal_count > 286 || distance_count > 30 {
        return Err(Stop::Invalid("too many length or distance symbols"));
    }
    let mut code_lengths = [0u8; 19];
    for place in ORDER.iter().take(code_count) {
        code_lengths[*place] = bits(reader.read_bits(3))? as u8;
    }
    if !Huffman::lengths_accepted(&code_lengths, false) {
        return Err(Stop::Invalid("invalid code lengths set"));
    }
    let code_code = Huffman::from_lengths(&code_lengths);
    let total = literal_count + distance_count;
    let mut lengths = Vec::with_capacity(total);
    while lengths.len() < total {
        let symbol = code_code.decode(reader).ok_or(Stop::Short)?;
        let (value, repeat) = match symbol {
            0..=15 => (symbol as u8, 1),
            16 => {
                let Some(previous) = lengths.last().copied() else {
                    return Err(Stop::Invalid("invalid bit length repeat"));
                };
                (previous, bits(reader.read_bits(2))? as usize + 3)
            }
            17 => (0, bits(reader.read_bits(3))? as usize + 3),
            _ => (0, bits(reader.read_bits(7))? as usize + 11),
        };
        if lengths.len() + repeat > total {
            return Err(Stop::Invalid("invalid bit length repeat"));
        }
        lengths.extend(std::iter::repeat_n(value, repeat));
    }
    if lengths[256] == 0 {
        return Err(Stop::Invalid("invalid code -- missing end-of-block"));
    }
    if !Huffman::lengths_accepted(&lengths[..literal_count], true) {
        return Err(Stop::Invalid("invalid literal/lengths set"));
    }
    if !Huffman::lengths_accepted(&lengths[literal_count..], true) {
        return Err(Stop::Invalid("invalid distances set"));
    }
    Ok((
        Huffman::from_lengths(&lengths[..literal_count]),
        Huffman::from_lengths(&lengths[literal_count..]),
    ))
}

/// What a DEFLATE stream stands for, or None when it is not one.
pub(crate) fn inflate(bytes: &[u8]) -> Option<Vec<u8>> {
    inflate_counted(bytes).map(|(held, _)| held)
}

/// What a DEFLATE stream stands for, alongside the number of bytes of it the
/// decoder read. Anything after that many bytes is not part of the stream.
pub(crate) fn inflate_counted(bytes: &[u8]) -> Option<(Vec<u8>, usize)> {
    inflate_counted_with(bytes, &[])
}

/// The same read, with a preset dictionary standing in front of the output so
/// a back-reference can reach into it.
pub(crate) fn inflate_counted_with(bytes: &[u8], dictionary: &[u8]) -> Option<(Vec<u8>, usize)> {
    inflate_checked_with(bytes, dictionary).ok().flatten()
}

/// The same read, telling a stream that stopped short, as None, from one
/// that is not a stream at all, as the reason zlib gives.
fn inflate_checked_with(
    bytes: &[u8],
    dictionary: &[u8],
) -> Result<Option<(Vec<u8>, usize)>, &'static str> {
    let mut out: Vec<u8> = dictionary.to_vec();
    match inflate_into(bytes, &mut out) {
        Ok(used) => {
            out.drain(..dictionary.len());
            Ok(Some((out, used)))
        }
        Err(Stop::Short) => Ok(None),
        Err(Stop::Invalid(reason)) => Err(reason),
    }
}

/// What the stream at the front of `bytes` stands for as far as the bytes
/// reach, whether or not the stream ends within them.
pub(crate) fn inflate_so_far(bytes: &[u8], dictionary: &[u8]) -> Vec<u8> {
    let mut out: Vec<u8> = dictionary.to_vec();
    let _ = inflate_into(bytes, &mut out);
    out.drain(..dictionary.len());
    out
}

/// Read a DEFLATE stream onto the end of `out`, answering how many bytes
/// the stream took. When the bytes run out first, `out` holds what was read
/// up to there and the stream stopped short.
fn inflate_into(bytes: &[u8], out: &mut Vec<u8>) -> Result<usize, Stop> {
    let mut reader = BitReader::new(bytes);
    loop {
        let last = bits(reader.read_bit())?;
        let kind = bits(reader.read_bits(2))?;
        match kind {
            0 => {
                reader.align();
                let at = reader.at;
                let header = bytes.get(at..at + 4).ok_or(Stop::Short)?;
                let length = u16::from_le_bytes([header[0], header[1]]);
                if length != !u16::from_le_bytes([header[2], header[3]]) {
                    return Err(Stop::Invalid("invalid stored block lengths"));
                }
                reader.at = at + 4;
                let length = length as usize;
                let available = bytes.len().saturating_sub(reader.at).min(length);
                out.extend_from_slice(&bytes[reader.at..reader.at + available]);
                if available < length {
                    return Err(Stop::Short);
                }
                reader.at += length;
            }
            1 | 2 => {
                let (literals, distances) = if kind == 1 {
                    fixed_codes()
                } else {
                    dynamic_codes(&mut reader)?
                };
                loop {
                    let symbol = literals.decode(&mut reader).ok_or(Stop::Short)?;
                    match symbol {
                        0..=255 => out.push(symbol as u8),
                        256 => break,
                        257..=285 => {
                            let slot = symbol as usize - 257;
                            let length = LENGTH_BASE[slot] as usize
                                + bits(reader.read_bits(LENGTH_EXTRA[slot]))? as usize;
                            let distance_symbol =
                                distances.decode(&mut reader).ok_or(Stop::Short)? as usize;
                            if distance_symbol >= DISTANCE_BASE.len() {
                                return Err(Stop::Invalid("invalid distance code"));
                            }
                            let distance = DISTANCE_BASE[distance_symbol] as usize
                                + bits(reader.read_bits(DISTANCE_EXTRA[distance_symbol]))? as usize;
                            if distance > out.len() {
                                return Err(Stop::Invalid("invalid distance too far back"));
                            }
                            let start = out.len() - distance;
                            for step in 0..length {
                                let byte = out[start + step];
                                out.push(byte);
                            }
                        }
                        _ => return Err(Stop::Invalid("invalid literal/length code")),
                    }
                }
            }
            _ => return Err(Stop::Invalid("invalid block type")),
        }
        if last == 1 {
            break;
        }
    }
    reader.align();
    Ok(reader.at)
}

/// Whether a zlib header says the stream was written against a dictionary the
/// reader has to be given before it can read the stream.
pub(crate) fn wants_dictionary(bytes: &[u8]) -> bool {
    bytes.len() >= 2 && bytes[1] & 0x20 != 0
}

/// What a zlib stream stands for, alongside the number of bytes it took up,
/// its header and checksum counted in, read against a preset dictionary the
/// header named. A stream that stopped short is None, and one that is not a
/// zlib stream is the reason zlib gives.
fn zlib_unwrap_checked_with(
    bytes: &[u8],
    dictionary: &[u8],
) -> Result<Option<(Vec<u8>, usize)>, &'static str> {
    if bytes.len() < 2 {
        return Ok(None);
    }
    if (u16::from(bytes[0]) << 8 | u16::from(bytes[1])) % 31 != 0 {
        return Err("incorrect header check");
    }
    if bytes[0] & 0x0f != 8 {
        return Err("unknown compression method");
    }
    if bytes[0] >> 4 > 7 {
        return Err("invalid window size");
    }
    // A header naming a dictionary is followed by the dictionary's checksum,
    // which the stream itself begins after.
    let front = if wants_dictionary(bytes) { 6 } else { 2 };
    let Some(body) = bytes.get(front..) else {
        return Ok(None);
    };
    let Some((held, used)) = inflate_checked_with(body, dictionary)? else {
        return Ok(None);
    };
    let Some(check) = bytes.get(front + used..front + used + 4) else {
        return Ok(None);
    };
    if u32::from_be_bytes([check[0], check[1], check[2], check[3]]) != adler32(1, &held) {
        return Err("incorrect data check");
    }
    Ok(Some((held, front + used + 4)))
}

/// The same, written against a dictionary. A stream that names one says so in
/// its header and carries the dictionary's checksum, so a reader can tell
/// whether it holds the dictionary the writer used.
pub(crate) fn zlib_wrap_with(bytes: &[u8], dictionary: &[u8], level: i64) -> Vec<u8> {
    // The header names how hard the writer worked, in the four steps zlib
    // sorts its levels into, and whether a dictionary was used, with a check
    // that makes the two bytes a multiple of 31.
    const CMF: u16 = 0x78;
    let effort: u16 = match level {
        0 | 1 => 0,
        2..=5 => 1,
        7..=9 => 3,
        _ => 2,
    };
    let mut flags = (effort << 6) | if dictionary.is_empty() { 0 } else { 0x20 };
    flags += 31 - ((CMF << 8) | flags) % 31;
    let mut out = vec![CMF as u8, flags as u8];
    if !dictionary.is_empty() {
        out.extend_from_slice(&adler32(1, dictionary).to_be_bytes());
    }
    out.extend_from_slice(&super::zlib_deflate::deflate(bytes, dictionary, level));
    out.extend_from_slice(&adler32(1, bytes).to_be_bytes());
    out
}

/// What a gzip member stands for, or the class and message of the error
/// Ruby's zlib raises for one that is not whole or not right: checked in the
/// order Ruby checks, the header, the compressed data, then the trailer's
/// CRC and length.
fn gzip_unwrap_checked(bytes: &[u8]) -> Result<Vec<u8>, (&'static str, String)> {
    const GZIP_ERROR: &str = "Zlib::GzipFile::Error";
    let short = || (GZIP_ERROR, "unexpected end of string".to_string());
    if bytes.len() < 2 {
        return Err(short());
    }
    if bytes[0] != 0x1f || bytes[1] != 0x8b {
        return Err((GZIP_ERROR, "not in gzip format".to_string()));
    }
    match bytes.get(2) {
        None => return Err(short()),
        Some(8) => {}
        Some(method) => {
            return Err((
                GZIP_ERROR,
                format!("unsupported compression method {method}"),
            ));
        }
    }
    let at = gzip_body_start(bytes).ok_or_else(short)?;
    let body = bytes.get(at..).ok_or_else(short)?;
    let (held, used) = match inflate_checked_with(body, &[]) {
        Ok(Some(read)) => read,
        Ok(None) => return Err((GZIP_ERROR, "unexpected end of file".to_string())),
        Err(reason) => return Err(("Zlib::DataError", reason.to_string())),
    };
    let trailer = bytes.get(at + used..at + used + 8).ok_or_else(short)?;
    let crc = u32::from_le_bytes([trailer[0], trailer[1], trailer[2], trailer[3]]);
    if crc != crc32(0, &held) {
        return Err((
            "Zlib::GzipFile::CRCError",
            "invalid compressed data -- crc error".to_string(),
        ));
    }
    let length = u32::from_le_bytes([trailer[4], trailer[5], trailer[6], trailer[7]]);
    if length != held.len() as u32 {
        return Err((
            "Zlib::GzipFile::LengthError",
            "invalid compressed data -- length error".to_string(),
        ));
    }
    Ok(held)
}

/// What a gzip stream at the front of `bytes` stands for, and how many bytes
/// it took, trailer included. None until the whole stream has arrived.
fn gzip_unwrap_counted(bytes: &[u8]) -> Option<(Vec<u8>, usize)> {
    let at = gzip_body_start(bytes)?;
    let (held, used) = inflate_counted_with(bytes.get(at..)?, &[])?;
    let ends = at + used + 8;
    if ends > bytes.len() {
        return None;
    }
    Some((held, ends))
}

/// Where the compressed body of a gzip stream starts, past the header and
/// the fields its flags name.
fn gzip_body_start(bytes: &[u8]) -> Option<usize> {
    if bytes.len() < 10 || bytes[0] != 0x1f || bytes[1] != 0x8b {
        return None;
    }
    let flags = bytes[3];
    let mut at = 10;
    if flags & 0x04 != 0 {
        let extra = u16::from_le_bytes([*bytes.get(at)?, *bytes.get(at + 1)?]) as usize;
        at += 2 + extra;
    }
    for mask in [0x08u8, 0x10] {
        if flags & mask != 0 {
            while *bytes.get(at)? != 0 {
                at += 1;
            }
            at += 1;
        }
    }
    if flags & 0x02 != 0 {
        at += 2;
    }
    Some(at)
}

/// Wrap a DEFLATE stream in the gzip header and trailer.
pub(crate) fn gzip_wrap(bytes: &[u8], name: Option<&str>, stamp: u32) -> Vec<u8> {
    let mut out = vec![0x1f, 0x8b, 0x08];
    out.push(if name.is_some() { 0x08 } else { 0x00 });
    out.extend_from_slice(&stamp.to_le_bytes());
    out.push(0);
    out.push(0x03);
    if let Some(name) = name {
        out.extend_from_slice(name.as_bytes());
        out.push(0);
    }
    out.extend_from_slice(&super::zlib_deflate::deflate(bytes, &[], -1));
    out.extend_from_slice(&crc32(0, bytes).to_le_bytes());
    out.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
    out
}

use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use crate::object::{OptionView, View};
use crate::vm::core::VirtualMachine;

impl VirtualMachine {
    /// `Zlib.__stream__(action, text, running)` — the one place the library
    /// reaches the encodings above, with bytes held one to a character.
    pub(crate) fn zlib_stream(
        &mut self,
        arguments: &[Object],
        position: Position,
    ) -> Result<Object, MetorexError> {
        let Some(View::String(action)) = arguments.first().view() else {
            return Err(crate::vm::errors::argument_count_error(
                crate::vm::errors::Arity::AtLeast(1),
                arguments.len(),
                position,
            ));
        };
        let text = match arguments.get(1).view() {
            Some(View::String(held)) => held.as_str().to_string(),
            _ => String::new(),
        };
        let bytes = super::pack_format::string_to_bytes(&text);
        let running = match arguments.get(2).view() {
            Some(View::Int(held)) => held as u32,
            _ => 0,
        };
        // The compression level a deflate is asked for, the default when
        // none is given.
        let level = match arguments.get(4).view() {
            Some(View::Int(held)) => held,
            _ => -1,
        };
        let refuse = |what: &str| {
            crate::vm::errors::simple_exception(
                "Zlib::DataError",
                &format!("{what} is not a stream this can read"),
                position,
            )
        };
        match &*action.as_str() {
            "crc32" => Ok(Object::Int(i64::from(crc32(running, &bytes)))),
            "adler32" => Ok(Object::Int(i64::from(adler32(running, &bytes)))),
            "crc_table" => Ok(Object::array(
                crc_table_values()
                    .into_iter()
                    .map(|held| Object::Int(i64::from(held)))
                    .collect(),
            )),
            "deflate" => {
                let dictionary = match arguments.get(3).view() {
                    Some(View::String(held)) => held.bytes().to_vec(),
                    _ => Vec::new(),
                };
                Ok(super::pack_format::bytes_to_string(&zlib_wrap_with(
                    &bytes,
                    &dictionary,
                    level,
                )))
            }
            // A piece of a raw stream written in parts, compressed with the
            // input before it in the window: one a sync flush ends, or the
            // last one.
            "deflate_piece" | "deflate_last_piece" => {
                let previous = match arguments.get(3).view() {
                    Some(View::String(held)) => super::pack_format::string_to_bytes(&held.as_str()),
                    _ => Vec::new(),
                };
                let last = &*action.as_str() == "deflate_last_piece";
                Ok(super::pack_format::bytes_to_string(
                    &super::zlib_deflate::deflate_piece(&bytes, &previous, last),
                ))
            }
            // How much of the stream zlib has handed on after reading what
            // was given with no flush asked for, header included.
            "deflate_handed_on" => {
                let dictionary = match arguments.get(3).view() {
                    Some(View::String(held)) => held.bytes().to_vec(),
                    _ => Vec::new(),
                };
                let header = if dictionary.is_empty() { 2 } else { 6 };
                let body = super::zlib_deflate::deflate_handed_on(&bytes, &dictionary, level);
                Ok(Object::Int((header + body) as i64))
            }
            "inflate" => match zlib_unwrap_checked_with(&bytes, &[]) {
                Ok(Some((held, _))) => Ok(super::pack_format::bytes_to_string(&held)),
                Ok(None) => Err(crate::vm::errors::simple_exception(
                    "Zlib::BufError",
                    "buffer error",
                    position,
                )),
                Err(reason) => Err(crate::vm::errors::simple_exception(
                    "Zlib::DataError",
                    reason,
                    position,
                )),
            },
            // What the stream stands for, and the bytes after it that are
            // not part of it, which `Zlib::Inflate` passes through.
            "inflate_part" | "raw_inflate_part" => {
                let dictionary = match arguments.get(3).view() {
                    Some(View::String(held)) => held.bytes().to_vec(),
                    _ => Vec::new(),
                };
                let raw = &*action.as_str() == "raw_inflate_part";
                if !raw && dictionary.is_empty() && wants_dictionary(&bytes) {
                    return Ok(Object::symbol("need_dictionary".to_string()));
                }
                let read = if raw {
                    inflate_checked_with(&bytes, &dictionary)
                } else {
                    zlib_unwrap_checked_with(&bytes, &dictionary)
                };
                match read {
                    Ok(Some((held, used))) => Ok(Object::array(vec![
                        super::pack_format::bytes_to_string(&held),
                        super::pack_format::bytes_to_string(&bytes[used.min(bytes.len())..]),
                    ])),
                    Ok(None) => Ok(Object::Nil),
                    Err(reason) => Err(crate::vm::errors::simple_exception(
                        "Zlib::DataError",
                        reason,
                        position,
                    )),
                }
            }
            // What a stream that has not all arrived stands for so far. The
            // text names the kind: `zlib`, `raw` or `gzip`.
            "inflate_so_far" => {
                let dictionary = match arguments.get(3).view() {
                    Some(View::String(held)) => held.bytes().to_vec(),
                    _ => Vec::new(),
                };
                let kind = arguments
                    .get(4)
                    .map(|held| held.to_string())
                    .unwrap_or_default();
                let front = match kind.as_str() {
                    "raw" => Some(0),
                    "gzip" => gzip_body_start(&bytes),
                    _ if bytes.len() < 2 => None,
                    _ if wants_dictionary(&bytes) => Some(6),
                    _ => Some(2),
                };
                let read = match front.and_then(|at| bytes.get(at..)) {
                    Some(body) => inflate_so_far(body, &dictionary),
                    None => Vec::new(),
                };
                Ok(super::pack_format::bytes_to_string(&read))
            }
            // A gzip stream read the way `inflate_part` reads a zlib one.
            "gzip_part" => match gzip_unwrap_counted(&bytes) {
                Some((held, used)) => Ok(Object::array(vec![
                    super::pack_format::bytes_to_string(&held),
                    super::pack_format::bytes_to_string(&bytes[used..]),
                ])),
                None => Ok(Object::Nil),
            },
            "raw_inflate" => match inflate(&bytes) {
                Some(held) => Ok(super::pack_format::bytes_to_string(&held)),
                None => Err(refuse("the text")),
            },
            "gzip" => {
                let name = match arguments.get(3).view() {
                    Some(View::String(held)) => Some(held.as_str().to_string()),
                    _ => None,
                };
                Ok(super::pack_format::bytes_to_string(&gzip_wrap(
                    &bytes,
                    name.as_deref(),
                    running,
                )))
            }
            "gunzip" => match gzip_unwrap_checked(&bytes) {
                Ok(held) => Ok(super::pack_format::bytes_to_string(&held)),
                Err((class, message)) => Err(crate::vm::errors::simple_exception(
                    class, &message, position,
                )),
            },
            other => Err(MetorexError::runtime_error(
                format!("unknown stream action {other}"),
                crate::vm::utils::position_to_location(position),
            )),
        }
    }
}
