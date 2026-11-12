//! Reading and writing the encodings that spell a character with one byte
//! or with a lead byte and a trail byte: GBK, GB18030, CP949, and EUC-KR. A
//! lead byte is read together with the byte after it only when that byte can
//! trail it. GB18030 also spells characters with four bytes, a lead, a digit,
//! a lead, and a digit.

use super::cp949_table::{CP949_DECODED, CP949_ENCODED};
use super::euc_kr_table::{EUC_KR_DECODED, EUC_KR_ENCODED};
use super::gb18030_table::{GB18030_DECODED, GB18030_ENCODED, GB18030_FOUR_BYTE_RUNS};
use super::gbk_table::{GBK_DECODED, GBK_ENCODED};
use super::string_methods::BytesProblem;

pub(crate) struct DoubleByte {
    leads: (u8, u8),
    trail_ranges: &'static [(u8, u8)],
    /// A byte above ASCII that spells a character by itself.
    lone_high_byte: Option<u8>,
    decoded: &'static [(u32, u32)],
    encoded: &'static [(u32, u32)],
    /// The runs of four-byte sequences below U+10000, for an encoding that
    /// has them.
    four_byte_runs: Option<&'static [(u32, u32, u32)]>,
}

/// The number of four-byte sequences between 81 30 81 30 and 90 30 81 30,
/// where GB18030 starts spelling the characters past U+FFFF in order.
const SEQUENCES_BELOW_THE_SUPPLEMENTARY_PLANES: u32 = 189000;

const DIGITS: (u8, u8) = (0x30, 0x39);

static GBK: DoubleByte = DoubleByte {
    leads: (0x81, 0xfe),
    trail_ranges: &[(0x40, 0x7e), (0x80, 0xfe)],
    lone_high_byte: Some(0x80),
    decoded: GBK_DECODED,
    encoded: GBK_ENCODED,
    four_byte_runs: None,
};

static GB18030: DoubleByte = DoubleByte {
    leads: (0x81, 0xfe),
    trail_ranges: &[(0x40, 0x7e), (0x80, 0xfe)],
    lone_high_byte: None,
    decoded: GB18030_DECODED,
    encoded: GB18030_ENCODED,
    four_byte_runs: Some(GB18030_FOUR_BYTE_RUNS),
};

static CP949: DoubleByte = DoubleByte {
    leads: (0x81, 0xfe),
    trail_ranges: &[(0x41, 0x5a), (0x61, 0x7a), (0x81, 0xfe)],
    lone_high_byte: Some(0x80),
    decoded: CP949_DECODED,
    encoded: CP949_ENCODED,
    four_byte_runs: None,
};

static EUC_KR: DoubleByte = DoubleByte {
    leads: (0xa1, 0xfe),
    trail_ranges: &[(0xa1, 0xfe)],
    lone_high_byte: None,
    decoded: EUC_KR_DECODED,
    encoded: EUC_KR_ENCODED,
    four_byte_runs: None,
};

/// The double-byte encoding an encoding name stands for.
pub(crate) fn double_byte(named: &str) -> Option<&'static DoubleByte> {
    match named {
        "GBK" | "CP936" => Some(&GBK),
        "GB18030" => Some(&GB18030),
        "CP949" => Some(&CP949),
        "EUC-KR" | "eucKR" => Some(&EUC_KR),
        _ => None,
    }
}

impl DoubleByte {
    fn opens_a_pair(&self, byte: u8) -> bool {
        (self.leads.0..=self.leads.1).contains(&byte)
    }

    fn ends_a_pair(&self, byte: u8) -> bool {
        self.trail_ranges
            .iter()
            .any(|(low, high)| (*low..=*high).contains(&byte))
    }

    fn is_digit(byte: Option<&u8>) -> bool {
        byte.is_some_and(|byte| (DIGITS.0..=DIGITS.1).contains(byte))
    }

    /// Whether the byte after a lead opens a four-byte sequence.
    fn opens_four_bytes(&self, bytes: &[u8], at: usize) -> bool {
        self.four_byte_runs.is_some() && Self::is_digit(bytes.get(at + 1))
    }

    fn is_whole_four_bytes(&self, bytes: &[u8], at: usize) -> bool {
        self.opens_a_pair(bytes[at])
            && self.opens_four_bytes(bytes, at)
            && bytes
                .get(at + 2)
                .is_some_and(|third| self.opens_a_pair(*third))
            && Self::is_digit(bytes.get(at + 3))
    }

    fn pair_width(&self, bytes: &[u8], at: usize) -> usize {
        if self.is_whole_four_bytes(bytes, at) {
            return 4;
        }
        match bytes.get(at + 1) {
            Some(&trail) if self.opens_a_pair(bytes[at]) && self.ends_a_pair(trail) => 2,
            _ => 1,
        }
    }

    /// Each character of a run of bytes.
    pub(crate) fn characters(&self, bytes: &[u8]) -> Vec<Vec<u8>> {
        let mut characters = Vec::new();
        let mut at = 0usize;
        while at < bytes.len() {
            let width = self.pair_width(bytes, at);
            characters.push(bytes[at..at + width].to_vec());
            at += width;
        }
        characters
    }

    /// Where a run of bytes stops spelling characters.
    pub(crate) fn problem(&self, bytes: &[u8]) -> Option<BytesProblem> {
        let mut at = 0usize;
        while at < bytes.len() {
            let byte = bytes[at];
            if byte < 0x80 || self.lone_high_byte == Some(byte) {
                at += 1;
                continue;
            }
            if !self.opens_a_pair(byte) {
                return Some((vec![byte], None, false));
            }
            if self.opens_four_bytes(bytes, at) {
                if self.is_whole_four_bytes(bytes, at) {
                    at += 4;
                    continue;
                }
                let read = &bytes[at..bytes.len().min(at + 4)];
                let incomplete =
                    read.len() < 4 && read.get(2).is_none_or(|third| self.opens_a_pair(*third));
                return Some((vec![byte], read.get(1).copied(), incomplete));
            }
            match bytes.get(at + 1) {
                None => return Some((vec![byte], None, true)),
                Some(&trail) if self.ends_a_pair(trail) => at += 2,
                Some(&trail) => return Some((vec![byte], Some(trail), false)),
            }
        }
        None
    }

    /// Whether a run of bytes spells characters.
    pub(crate) fn reads(&self, bytes: &[u8]) -> bool {
        self.problem(bytes).is_none()
    }

    /// The character a run of bytes opens with, alongside how many bytes it
    /// took. None where the run opens no character the encoding names.
    pub(crate) fn character(&self, bytes: &[u8]) -> Option<(char, usize)> {
        let first = *bytes.first()?;
        if first < 0x80 {
            return Some((char::from(first), 1));
        }
        let width = self.pair_width(bytes, 0);
        if width == 4 {
            return self.four_byte_character(&bytes[..4]).map(|held| (held, 4));
        }
        let packed = bytes[..width]
            .iter()
            .fold(0u32, |packed, byte| (packed << 8) | u32::from(*byte));
        let at = self
            .decoded
            .binary_search_by_key(&packed, |(held, _)| *held)
            .ok()?;
        char::from_u32(self.decoded[at].1).map(|held| (held, width))
    }

    /// The character a whole four-byte sequence stands for.
    fn four_byte_character(&self, bytes: &[u8]) -> Option<char> {
        let runs = self.four_byte_runs?;
        let index = ((u32::from(bytes[0]) - 0x81) * 10 + u32::from(bytes[1]) - 0x30) * 1260
            + (u32::from(bytes[2]) - 0x81) * 10
            + u32::from(bytes[3])
            - 0x30;
        if index >= SEQUENCES_BELOW_THE_SUPPLEMENTARY_PLANES {
            return char::from_u32(index - SEQUENCES_BELOW_THE_SUPPLEMENTARY_PLANES + 0x10000);
        }
        let at = runs
            .partition_point(|(start, _, _)| *start <= index)
            .checked_sub(1)?;
        let (start, point, length) = runs[at];
        (index < start + length)
            .then(|| char::from_u32(point + index - start))
            .flatten()
    }

    /// The four bytes a character is spelled with, for an encoding that has
    /// them.
    fn four_bytes_for(&self, point: u32) -> Option<[u8; 4]> {
        let runs = self.four_byte_runs?;
        let index = if point >= 0x10000 {
            point - 0x10000 + SEQUENCES_BELOW_THE_SUPPLEMENTARY_PLANES
        } else {
            let at = runs
                .partition_point(|(_, first, _)| *first <= point)
                .checked_sub(1)?;
            let (start, first, length) = runs[at];
            if point >= first + length {
                return None;
            }
            start + point - first
        };
        let fourth = index % 10;
        let third = index / 10 % 126;
        let second = index / 1260 % 10;
        let first = index / 12600;
        Some([
            (0x81 + first) as u8,
            (0x30 + second) as u8,
            (0x81 + third) as u8,
            (0x30 + fourth) as u8,
        ])
    }

    /// The text a run of bytes spells. A byte that opens no character stands
    /// for itself.
    pub(crate) fn text(&self, bytes: &[u8]) -> String {
        self.text_replacing(bytes, None)
    }

    /// The text a run of bytes spells, with `replacement`, when given, in
    /// place of each character the encoding names no character for.
    pub(crate) fn text_replacing(&self, bytes: &[u8], replacement: Option<&str>) -> String {
        let mut written = String::with_capacity(bytes.len());
        let mut at = 0usize;
        while at < bytes.len() {
            match (self.character(&bytes[at..]), replacement) {
                (Some((character, width)), _) => {
                    written.push(character);
                    at += width;
                }
                (None, Some(replacement)) if self.spells_a_whole_character(bytes, at) => {
                    written.push_str(replacement);
                    at += self.pair_width(bytes, at);
                }
                (None, _) => {
                    written.push(char::from(bytes[at]));
                    at += 1;
                }
            }
        }
        written
    }

    /// The bytes of the first character a run spells that the encoding has
    /// no character for.
    pub(crate) fn undefined(&self, bytes: &[u8]) -> Option<Vec<u8>> {
        let mut at = 0usize;
        while at < bytes.len() {
            let width = self.pair_width(bytes, at);
            if self.character(&bytes[at..]).is_none() && self.spells_a_whole_character(bytes, at) {
                return Some(bytes[at..at + width].to_vec());
            }
            at += width;
        }
        None
    }

    /// Whether the bytes at `at` make a whole character, which a lone lead
    /// byte does not.
    fn spells_a_whole_character(&self, bytes: &[u8], at: usize) -> bool {
        self.pair_width(bytes, at) > 1 || self.lone_high_byte == Some(bytes[at])
    }

    /// The bytes the encoding spells a whole text with, or the first
    /// character it has no bytes for.
    pub(crate) fn bytes(&self, text: &str) -> Result<Vec<u8>, char> {
        let mut bytes = Vec::with_capacity(text.len());
        for character in text.chars() {
            let point = character as u32;
            if point < 0x80 {
                bytes.push(point as u8);
                continue;
            }
            let Ok(at) = self.encoded.binary_search_by_key(&point, |(held, _)| *held) else {
                let spelled = self.four_bytes_for(point).ok_or(character)?;
                bytes.extend_from_slice(&spelled);
                continue;
            };
            let packed = self.encoded[at].1;
            if packed > 0xff {
                bytes.push((packed >> 8) as u8);
            }
            bytes.push((packed & 0xff) as u8);
        }
        Ok(bytes)
    }
}
