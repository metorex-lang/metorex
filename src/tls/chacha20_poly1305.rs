// ChaCha20 and Poly1305 as an AEAD, as RFC 8439 defines it, with the
// 12-byte nonces and 16-byte tags TLS 1.3 records use.

const TAG: usize = 16;
const LIMB_MASK: u32 = 0x3ff_ffff;

/// A 32-byte key, for sealing and opening many records.
pub struct ChaCha20Poly1305 {
    key: [u32; 8],
}

fn little_endian(bytes: &[u8]) -> u32 {
    u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
}

fn quarter_round(state: &mut [u32; 16], a: usize, b: usize, c: usize, d: usize) {
    state[a] = state[a].wrapping_add(state[b]);
    state[d] = (state[d] ^ state[a]).rotate_left(16);
    state[c] = state[c].wrapping_add(state[d]);
    state[b] = (state[b] ^ state[c]).rotate_left(12);
    state[a] = state[a].wrapping_add(state[b]);
    state[d] = (state[d] ^ state[a]).rotate_left(8);
    state[c] = state[c].wrapping_add(state[d]);
    state[b] = (state[b] ^ state[c]).rotate_left(7);
}

/// Poly1305 over 26-bit limbs, the accumulator `h` multiplied by `r` for
/// each 16-byte block and reduced modulo 2^130 - 5.
struct Poly1305 {
    r: [u32; 5],
    h: [u32; 5],
    pad: [u32; 4],
}

impl Poly1305 {
    fn new(key: &[u8]) -> Poly1305 {
        Poly1305 {
            r: [
                little_endian(&key[0..]) & 0x3ff_ffff,
                (little_endian(&key[3..]) >> 2) & 0x3ff_ff03,
                (little_endian(&key[6..]) >> 4) & 0x3ff_c0ff,
                (little_endian(&key[9..]) >> 6) & 0x3f0_3fff,
                (little_endian(&key[12..]) >> 8) & 0x00f_ffff,
            ],
            h: [0; 5],
            pad: [
                little_endian(&key[16..]),
                little_endian(&key[20..]),
                little_endian(&key[24..]),
                little_endian(&key[28..]),
            ],
        }
    }

    /// Absorb `data` zero-padded to whole blocks, as the AEAD's MAC input
    /// pads each of its parts.
    fn padded(&mut self, data: &[u8]) {
        for chunk in data.chunks(16) {
            let mut block = [0u8; 16];
            block[..chunk.len()].copy_from_slice(chunk);
            self.block(&block);
        }
    }

    fn block(&mut self, block: &[u8; 16]) {
        let [r0, r1, r2, r3, r4] = self.r.map(u64::from);
        let (s1, s2, s3, s4) = (r1 * 5, r2 * 5, r3 * 5, r4 * 5);
        let h0 = u64::from(self.h[0] + (little_endian(&block[0..]) & LIMB_MASK));
        let h1 = u64::from(self.h[1] + ((little_endian(&block[3..]) >> 2) & LIMB_MASK));
        let h2 = u64::from(self.h[2] + ((little_endian(&block[6..]) >> 4) & LIMB_MASK));
        let h3 = u64::from(self.h[3] + ((little_endian(&block[9..]) >> 6) & LIMB_MASK));
        let h4 = u64::from(self.h[4] + ((little_endian(&block[12..]) >> 8) | (1 << 24)));

        let d0 = h0 * r0 + h1 * s4 + h2 * s3 + h3 * s2 + h4 * s1;
        let mut d1 = h0 * r1 + h1 * r0 + h2 * s4 + h3 * s3 + h4 * s2;
        let mut d2 = h0 * r2 + h1 * r1 + h2 * r0 + h3 * s4 + h4 * s3;
        let mut d3 = h0 * r3 + h1 * r2 + h2 * r1 + h3 * r0 + h4 * s4;
        let mut d4 = h0 * r4 + h1 * r3 + h2 * r2 + h3 * r1 + h4 * r0;

        let mask = u64::from(LIMB_MASK);
        d1 += d0 >> 26;
        d2 += d1 >> 26;
        d3 += d2 >> 26;
        d4 += d3 >> 26;
        let mut first = (d0 & mask) + (d4 >> 26) * 5;
        let second = (d1 & mask) + (first >> 26);
        first &= mask;
        self.h = [
            first as u32,
            second as u32,
            (d2 & mask) as u32,
            (d3 & mask) as u32,
            (d4 & mask) as u32,
        ];
    }

    fn tag(mut self) -> [u8; TAG] {
        let h = &mut self.h;
        for at in 1..5 {
            h[at] += h[at - 1] >> 26;
            h[at - 1] &= LIMB_MASK;
        }
        h[0] += (h[4] >> 26) * 5;
        h[4] &= LIMB_MASK;
        h[1] += h[0] >> 26;
        h[0] &= LIMB_MASK;

        // h - p, kept when it does not go below zero.
        let mut g = [0u32; 5];
        let mut carry = 5;
        for at in 0..5 {
            let sum = h[at] + carry;
            carry = sum >> 26;
            g[at] = sum & LIMB_MASK;
        }
        g[4] = g[4].wrapping_add(carry << 26).wrapping_sub(1 << 26);
        let keep_g = (g[4] >> 31).wrapping_sub(1);
        for at in 0..5 {
            h[at] = (h[at] & !keep_g) | (g[at] & keep_g);
        }

        let words = [
            h[0] | (h[1] << 26),
            (h[1] >> 6) | (h[2] << 20),
            (h[2] >> 12) | (h[3] << 14),
            (h[3] >> 18) | (h[4] << 8),
        ];
        let mut tag = [0u8; TAG];
        let mut carry = 0u64;
        for at in 0..4 {
            let sum = u64::from(words[at]) + u64::from(self.pad[at]) + carry;
            carry = sum >> 32;
            tag[at * 4..at * 4 + 4].copy_from_slice(&(sum as u32).to_le_bytes());
        }
        tag
    }
}

impl ChaCha20Poly1305 {
    pub fn new(key: &[u8]) -> Option<ChaCha20Poly1305> {
        if key.len() != 32 {
            return None;
        }
        let mut words = [0u32; 8];
        for (at, word) in words.iter_mut().enumerate() {
            *word = little_endian(&key[at * 4..]);
        }
        Some(ChaCha20Poly1305 { key: words })
    }

    /// The 64-byte ChaCha20 key stream block at `counter`.
    fn key_stream(&self, counter: u32, nonce: &[u8; 12]) -> [u8; 64] {
        let mut state = [0u32; 16];
        state[..4].copy_from_slice(&[0x6170_7865, 0x3320_646e, 0x7962_2d32, 0x6b20_6574]);
        state[4..12].copy_from_slice(&self.key);
        state[12] = counter;
        for at in 0..3 {
            state[13 + at] = little_endian(&nonce[at * 4..]);
        }
        let mut working = state;
        for _ in 0..10 {
            quarter_round(&mut working, 0, 4, 8, 12);
            quarter_round(&mut working, 1, 5, 9, 13);
            quarter_round(&mut working, 2, 6, 10, 14);
            quarter_round(&mut working, 3, 7, 11, 15);
            quarter_round(&mut working, 0, 5, 10, 15);
            quarter_round(&mut working, 1, 6, 11, 12);
            quarter_round(&mut working, 2, 7, 8, 13);
            quarter_round(&mut working, 3, 4, 9, 14);
        }
        let mut stream = [0u8; 64];
        for at in 0..16 {
            let word = working[at].wrapping_add(state[at]);
            stream[at * 4..at * 4 + 4].copy_from_slice(&word.to_le_bytes());
        }
        stream
    }

    fn apply_key_stream(&self, nonce: &[u8; 12], data: &mut [u8]) {
        for (index, chunk) in data.chunks_mut(64).enumerate() {
            let stream = self.key_stream(index as u32 + 1, nonce);
            for (byte, mask) in chunk.iter_mut().zip(stream) {
                *byte ^= mask;
            }
        }
    }

    fn tag(&self, nonce: &[u8; 12], additional: &[u8], ciphertext: &[u8]) -> [u8; TAG] {
        let mut mac = Poly1305::new(&self.key_stream(0, nonce)[..32]);
        mac.padded(additional);
        mac.padded(ciphertext);
        let mut lengths = [0u8; 16];
        lengths[..8].copy_from_slice(&(additional.len() as u64).to_le_bytes());
        lengths[8..].copy_from_slice(&(ciphertext.len() as u64).to_le_bytes());
        mac.block(&lengths);
        mac.tag()
    }

    /// The plaintext encrypted, with its tag after it.
    pub fn seal(&self, nonce: &[u8; 12], additional: &[u8], plaintext: &[u8]) -> Vec<u8> {
        let mut sealed = plaintext.to_vec();
        self.apply_key_stream(nonce, &mut sealed);
        let tag = self.tag(nonce, additional, &sealed);
        sealed.extend_from_slice(&tag);
        sealed
    }

    /// The plaintext of a sealed record, or None when its tag does not
    /// match what it holds.
    pub fn open(&self, nonce: &[u8; 12], additional: &[u8], sealed: &[u8]) -> Option<Vec<u8>> {
        if sealed.len() < TAG {
            return None;
        }
        let (ciphertext, tag) = sealed.split_at(sealed.len() - TAG);
        let expected = self.tag(nonce, additional, ciphertext);
        let differing = expected
            .iter()
            .zip(tag)
            .fold(0u8, |held, (left, right)| held | (left ^ right));
        if differing != 0 {
            return None;
        }
        let mut plaintext = ciphertext.to_vec();
        self.apply_key_stream(nonce, &mut plaintext);
        Some(plaintext)
    }
}
