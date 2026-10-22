// AES in Galois/Counter Mode, as NIST SP 800-38D defines it, with the
// 12-byte nonces and 16-byte tags TLS 1.3 records use.

use crate::vm::native_methods::aes::{encrypt_block, expand_key};

const BLOCK: usize = 16;

/// A key expanded once, for sealing and opening many records.
pub struct Gcm {
    round_keys: Vec<[u8; BLOCK]>,
    hash_key: u128,
}

fn block_value(bytes: &[u8]) -> u128 {
    let mut held = [0u8; BLOCK];
    held[..bytes.len()].copy_from_slice(bytes);
    u128::from_be_bytes(held)
}

/// Multiplication in GF(2^128) with the bit order GCM uses.
fn multiply(left: u128, right: u128) -> u128 {
    const REDUCTION: u128 = 0xE1 << 120;
    let mut product = 0u128;
    let mut shifted = right;
    for bit in 0..128 {
        if (left >> (127 - bit)) & 1 == 1 {
            product ^= shifted;
        }
        let carried = shifted & 1 == 1;
        shifted >>= 1;
        if carried {
            shifted ^= REDUCTION;
        }
    }
    product
}

impl Gcm {
    pub fn new(key: &[u8]) -> Option<Gcm> {
        let round_keys = expand_key(key)?;
        let mut zero = [0u8; BLOCK];
        encrypt_block(&mut zero, &round_keys);
        Some(Gcm {
            round_keys,
            hash_key: u128::from_be_bytes(zero),
        })
    }

    fn ghash(&self, additional: &[u8], ciphertext: &[u8]) -> u128 {
        let mut state = 0u128;
        for chunk in additional.chunks(BLOCK) {
            state = multiply(state ^ block_value(chunk), self.hash_key);
        }
        for chunk in ciphertext.chunks(BLOCK) {
            state = multiply(state ^ block_value(chunk), self.hash_key);
        }
        let lengths = ((additional.len() as u128 * 8) << 64) | (ciphertext.len() as u128 * 8);
        multiply(state ^ lengths, self.hash_key)
    }

    /// The counter block for a nonce, numbered from `counter`.
    fn counter_block(nonce: &[u8; 12], counter: u32) -> [u8; BLOCK] {
        let mut block = [0u8; BLOCK];
        block[..12].copy_from_slice(nonce);
        block[12..].copy_from_slice(&counter.to_be_bytes());
        block
    }

    fn apply_counter(&self, nonce: &[u8; 12], data: &mut [u8]) {
        for (index, chunk) in data.chunks_mut(BLOCK).enumerate() {
            let mut stream = Gcm::counter_block(nonce, 2 + index as u32);
            encrypt_block(&mut stream, &self.round_keys);
            for (byte, key) in chunk.iter_mut().zip(stream) {
                *byte ^= key;
            }
        }
    }

    fn tag(&self, nonce: &[u8; 12], additional: &[u8], ciphertext: &[u8]) -> [u8; BLOCK] {
        let mut first = Gcm::counter_block(nonce, 1);
        encrypt_block(&mut first, &self.round_keys);
        (self.ghash(additional, ciphertext) ^ u128::from_be_bytes(first)).to_be_bytes()
    }

    /// The plaintext encrypted, with its tag after it.
    pub fn seal(&self, nonce: &[u8; 12], additional: &[u8], plaintext: &[u8]) -> Vec<u8> {
        let mut sealed = plaintext.to_vec();
        self.apply_counter(nonce, &mut sealed);
        let tag = self.tag(nonce, additional, &sealed);
        sealed.extend_from_slice(&tag);
        sealed
    }

    /// The plaintext of a sealed record, or None when its tag does not
    /// match what it holds.
    pub fn open(&self, nonce: &[u8; 12], additional: &[u8], sealed: &[u8]) -> Option<Vec<u8>> {
        if sealed.len() < BLOCK {
            return None;
        }
        let (ciphertext, tag) = sealed.split_at(sealed.len() - BLOCK);
        let expected = self.tag(nonce, additional, ciphertext);
        let differing = expected
            .iter()
            .zip(tag)
            .fold(0u8, |held, (left, right)| held | (left ^ right));
        if differing != 0 {
            return None;
        }
        let mut plaintext = ciphertext.to_vec();
        self.apply_counter(nonce, &mut plaintext);
        Some(plaintext)
    }
}
