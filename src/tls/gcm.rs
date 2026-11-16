// AES in Galois/Counter Mode, as NIST SP 800-38D defines it, with the
// 12-byte nonces and 16-byte tags TLS 1.3 records use.

use crate::vm::native_methods::aes::{encrypt_block, expand_key};

const BLOCK: usize = 16;

/// A key expanded once, for sealing and opening many records.
pub struct Gcm {
    round_keys: Vec<[u8; BLOCK]>,
    /// The hash key times each 4-bit value, whose first bit is x^0.
    hash_multiples: [u128; 16],
}

fn block_value(bytes: &[u8]) -> u128 {
    let mut held = [0u8; BLOCK];
    held[..bytes.len()].copy_from_slice(bytes);
    u128::from_be_bytes(held)
}

/// The value times x in GF(2^128) with the bit order GCM uses, where
/// each step toward the low end of the block is one power of x higher.
fn times_x(value: u128) -> u128 {
    const REDUCTION: u128 = 0xE1 << 120;
    if value & 1 == 1 {
        (value >> 1) ^ REDUCTION
    } else {
        value >> 1
    }
}

/// What the four bits shifted off the low end of a value carry back in when
/// it is multiplied by x^4, by those bits.
const FOLDED_BACK: [u128; 16] = {
    let mut table = [0u128; 16];
    let mut bits = 0;
    while bits < 16 {
        let mut value = bits as u128;
        let mut step = 0;
        while step < 4 {
            const REDUCTION: u128 = 0xE1 << 120;
            value = if value & 1 == 1 {
                (value >> 1) ^ REDUCTION
            } else {
                value >> 1
            };
            step += 1;
        }
        table[bits] = value;
        bits += 1;
    }
    table
};

impl Gcm {
    pub fn new(key: &[u8]) -> Option<Gcm> {
        let round_keys = expand_key(key)?;
        let mut zero = [0u8; BLOCK];
        encrypt_block(&mut zero, &round_keys);
        let mut hash_multiples = [0u128; 16];
        hash_multiples[8] = u128::from_be_bytes(zero);
        for bit in [4, 2, 1] {
            hash_multiples[bit] = times_x(hash_multiples[bit * 2]);
        }
        for value in 1..16 {
            let high = 1 << (u32::BITS - 1 - (value as u32).leading_zeros());
            hash_multiples[value] = hash_multiples[high] ^ hash_multiples[value ^ high];
        }
        Some(Gcm {
            round_keys,
            hash_multiples,
        })
    }

    /// The value times the hash key, four bits at a time from the highest
    /// power of x down.
    fn times_hash_key(&self, value: u128) -> u128 {
        let mut product = 0u128;
        for nibble in 0..32 {
            let bits = (value >> (4 * nibble)) & 0xF;
            product = (product >> 4) ^ FOLDED_BACK[(product & 0xF) as usize];
            product ^= self.hash_multiples[bits as usize];
        }
        product
    }

    fn ghash(&self, additional: &[u8], ciphertext: &[u8]) -> u128 {
        let mut state = 0u128;
        for chunk in additional.chunks(BLOCK) {
            state = self.times_hash_key(state ^ block_value(chunk));
        }
        for chunk in ciphertext.chunks(BLOCK) {
            state = self.times_hash_key(state ^ block_value(chunk));
        }
        let lengths = ((additional.len() as u128 * 8) << 64) | (ciphertext.len() as u128 * 8);
        self.times_hash_key(state ^ lengths)
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
