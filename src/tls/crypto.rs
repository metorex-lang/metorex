// The key derivation and signatures TLS 1.3 runs on: HKDF over SHA-256
// (RFC 5869, with TLS 1.3's labels), and RSA signatures in PSS and in
// PKCS #1 v1.5 form (RFC 8017).

use num_bigint::BigUint;

use crate::vm::native_methods::digest_algorithms::{hmac, sha256};

pub const HASH_LENGTH: usize = 32;

pub fn hkdf_extract(salt: &[u8], keying: &[u8]) -> Vec<u8> {
    let salt = if salt.is_empty() {
        vec![0u8; HASH_LENGTH]
    } else {
        salt.to_vec()
    };
    hmac("SHA256", &salt, keying).unwrap_or_default()
}

pub fn hkdf_expand(secret: &[u8], info: &[u8], length: usize) -> Vec<u8> {
    let mut output = Vec::with_capacity(length);
    let mut previous = Vec::new();
    let mut counter = 1u8;
    while output.len() < length {
        let mut input = previous.clone();
        input.extend_from_slice(info);
        input.push(counter);
        previous = hmac("SHA256", secret, &input).unwrap_or_default();
        output.extend_from_slice(&previous);
        counter += 1;
    }
    output.truncate(length);
    output
}

/// HKDF-Expand-Label, which writes the label and context the way TLS 1.3
/// asks.
pub fn expand_label(secret: &[u8], label: &str, context: &[u8], length: usize) -> Vec<u8> {
    let full = format!("tls13 {label}");
    let mut info = Vec::new();
    info.extend_from_slice(&(length as u16).to_be_bytes());
    info.push(full.len() as u8);
    info.extend_from_slice(full.as_bytes());
    info.push(context.len() as u8);
    info.extend_from_slice(context);
    hkdf_expand(secret, &info, length)
}

pub fn derive_secret(secret: &[u8], label: &str, transcript: &[u8]) -> Vec<u8> {
    expand_label(secret, label, &sha256(transcript), HASH_LENGTH)
}

/// An RSA key, its private exponent present when it can sign.
#[derive(Clone, Debug)]
pub struct RsaKey {
    pub modulus: BigUint,
    pub public_exponent: BigUint,
    pub private_exponent: Option<BigUint>,
}

impl RsaKey {
    fn length(&self) -> usize {
        self.modulus.bits().div_ceil(8) as usize
    }

    fn bytes_of(&self, value: &BigUint) -> Vec<u8> {
        let bytes = value.to_bytes_be();
        let mut padded = vec![0u8; self.length().saturating_sub(bytes.len())];
        padded.extend_from_slice(&bytes);
        padded
    }

    fn private_operation(&self, message: &[u8]) -> Option<Vec<u8>> {
        let exponent = self.private_exponent.as_ref()?;
        let value = BigUint::from_bytes_be(message);
        Some(self.bytes_of(&value.modpow(exponent, &self.modulus)))
    }

    fn public_operation(&self, signature: &[u8]) -> Option<Vec<u8>> {
        if signature.len() != self.length() {
            return None;
        }
        let value = BigUint::from_bytes_be(signature);
        if value >= self.modulus {
            return None;
        }
        Some(self.bytes_of(&value.modpow(&self.public_exponent, &self.modulus)))
    }

    /// An RSASSA-PSS signature over SHA-256, salted with as many bytes as
    /// the hash, as TLS 1.3's rsa_pss_rsae_sha256 asks.
    pub fn sign_pss(&self, message: &[u8], salt: &[u8]) -> Option<Vec<u8>> {
        let bits = self.modulus.bits() as usize - 1;
        let encoded_length = bits.div_ceil(8);
        let digest = sha256(message);
        let mut salted = vec![0u8; 8];
        salted.extend_from_slice(&digest);
        salted.extend_from_slice(salt);
        let hashed = sha256(&salted);
        let mut block = vec![0u8; encoded_length - salt.len() - HASH_LENGTH - 2];
        block.push(1);
        block.extend_from_slice(salt);
        let mask = mgf1(&hashed, block.len());
        for (byte, masking) in block.iter_mut().zip(mask) {
            *byte ^= masking;
        }
        block[0] &= 0xFF >> (8 * encoded_length - bits);
        block.extend_from_slice(&hashed);
        block.push(0xBC);
        self.private_operation(&block)
    }

    pub fn verify_pss(&self, message: &[u8], signature: &[u8]) -> bool {
        let Some(decoded) = self.public_operation(signature) else {
            return false;
        };
        let bits = self.modulus.bits() as usize - 1;
        let encoded_length = bits.div_ceil(8);
        let encoded = &decoded[decoded.len() - encoded_length..];
        if encoded.last() != Some(&0xBC) || encoded_length < HASH_LENGTH * 2 + 2 {
            return false;
        }
        let (masked, rest) = encoded.split_at(encoded_length - HASH_LENGTH - 1);
        let hashed = &rest[..HASH_LENGTH];
        let mut block: Vec<u8> = masked
            .iter()
            .zip(mgf1(hashed, masked.len()))
            .map(|(byte, masking)| byte ^ masking)
            .collect();
        block[0] &= 0xFF >> (8 * encoded_length - bits);
        let Some(marker) = block.iter().position(|byte| *byte != 0) else {
            return false;
        };
        if block[marker] != 1 {
            return false;
        }
        let salt = &block[marker + 1..];
        let mut salted = vec![0u8; 8];
        salted.extend_from_slice(&sha256(message));
        salted.extend_from_slice(salt);
        sha256(&salted) == hashed
    }

    /// An RSASSA-PKCS1-v1_5 signature over the digest `digest`, which the
    /// DigestInfo `prefix` names the hash of.
    pub fn sign_pkcs1(&self, prefix: &[u8], digest: &[u8]) -> Option<Vec<u8>> {
        let length = self.length();
        let mut block = vec![0u8, 1];
        block.extend(std::iter::repeat_n(
            0xFF,
            length - prefix.len() - digest.len() - 3,
        ));
        block.push(0);
        block.extend_from_slice(prefix);
        block.extend_from_slice(digest);
        self.private_operation(&block)
    }

    pub fn verify_pkcs1(&self, prefix: &[u8], digest: &[u8], signature: &[u8]) -> bool {
        let Some(decoded) = self.public_operation(signature) else {
            return false;
        };
        let length = self.length();
        if length < prefix.len() + digest.len() + 11 {
            return false;
        }
        let mut expected = vec![0u8, 1];
        expected.extend(std::iter::repeat_n(
            0xFF,
            length - prefix.len() - digest.len() - 3,
        ));
        expected.push(0);
        expected.extend_from_slice(prefix);
        expected.extend_from_slice(digest);
        decoded == expected
    }
}

/// MGF1 over SHA-256.
fn mgf1(seed: &[u8], length: usize) -> Vec<u8> {
    let mut output = Vec::with_capacity(length);
    let mut counter = 0u32;
    while output.len() < length {
        let mut input = seed.to_vec();
        input.extend_from_slice(&counter.to_be_bytes());
        output.extend_from_slice(&sha256(&input));
        counter += 1;
    }
    output.truncate(length);
    output
}

/// The DigestInfo prefix of a SHA-256 digest in a PKCS #1 v1.5 signature.
pub const SHA256_DIGEST_INFO: [u8; 19] = [
    0x30, 0x31, 0x30, 0x0d, 0x06, 0x09, 0x60, 0x86, 0x48, 0x01, 0x65, 0x03, 0x04, 0x02, 0x01, 0x05,
    0x00, 0x04, 0x20,
];
