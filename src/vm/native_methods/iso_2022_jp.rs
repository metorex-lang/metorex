//! Reading ISO-2022-JP as the converter to UTF-8 does: ASCII until an escape
//! sequence switches to JIS X 0208, where each character is a pair of bytes
//! from 0x21 to 0x7E, until another switches back.

use super::euc_jp_table::euc_jp_character;

const ESCAPE: u8 = 0x1b;
const SHIFT_OUT: u8 = 0x0e;
const SHIFT_IN: u8 = 0x0f;

/// One piece of a run of ISO-2022-JP bytes.
pub(crate) enum JisPiece {
    Character(char),
    /// Bytes that spell nothing, with the byte after them that ended the try
    /// and is read again, and whether the run ended before they were whole.
    Broken {
        bytes: Vec<u8>,
        after: Option<u8>,
        incomplete: bool,
    },
    /// A pair JIS X 0208 has no character for, as the EUC-JP bytes the
    /// conversion reads it as.
    Undefined(Vec<u8>),
}

fn is_jis_byte(byte: u8) -> bool {
    (0x21..=0x7e).contains(&byte)
}

/// The pieces a run of ISO-2022-JP bytes reads as.
pub(crate) fn iso_2022_jp_pieces(bytes: &[u8]) -> Vec<JisPiece> {
    let mut pieces = Vec::new();
    let mut in_japanese = false;
    let mut at = 0usize;
    let broken = |from: usize, to: usize, after: Option<u8>, incomplete: bool| JisPiece::Broken {
        bytes: bytes[from..to].to_vec(),
        after,
        incomplete,
    };
    while at < bytes.len() {
        let byte = bytes[at];
        if byte == ESCAPE {
            let (finals, intermediate): (&[u8], Option<&u8>) = match bytes.get(at + 1) {
                Some(b'$') => (b"@B", Some(&b'$')),
                Some(b'(') => (b"BJ", Some(&b'(')),
                Some(other) => {
                    pieces.push(broken(at, at + 1, Some(*other), false));
                    at += 1;
                    continue;
                }
                None => {
                    pieces.push(broken(at, at + 1, None, true));
                    break;
                }
            };
            match bytes.get(at + 2) {
                Some(last) if finals.contains(last) => {
                    in_japanese = intermediate == Some(&b'$');
                    at += 3;
                }
                Some(other) => {
                    pieces.push(broken(at, at + 2, Some(*other), false));
                    at += 2;
                }
                None => {
                    pieces.push(broken(at, bytes.len(), None, true));
                    break;
                }
            }
            continue;
        }
        if byte >= 0x80 || byte == SHIFT_OUT || byte == SHIFT_IN {
            pieces.push(broken(at, at + 1, None, false));
            at += 1;
            continue;
        }
        if !in_japanese {
            pieces.push(JisPiece::Character(char::from(byte)));
            at += 1;
            continue;
        }
        if !is_jis_byte(byte) {
            pieces.push(broken(at, at + 1, None, false));
            at += 1;
            continue;
        }
        match bytes.get(at + 1) {
            None => {
                pieces.push(broken(at, at + 1, None, true));
                break;
            }
            Some(trail) if !is_jis_byte(*trail) => {
                pieces.push(broken(at, at + 1, Some(*trail), false));
                at += 1;
            }
            Some(trail) => {
                let pair = [byte + 0x80, trail + 0x80];
                pieces.push(match euc_jp_character(&pair) {
                    Some((character, _)) => JisPiece::Character(character),
                    None => JisPiece::Undefined(pair.to_vec()),
                });
                at += 2;
            }
        }
    }
    pieces
}
