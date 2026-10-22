// The parts of an X.509 certificate the handshake reads itself: the RSA
// key it names, to check the peer's CertificateVerify against. Whether the
// chain is trusted is decided by the program's OpenSSL::X509::Store.

use num_bigint::BigUint;

use super::crypto::RsaKey;

/// One DER element: its tag, its contents, and the bytes after it.
fn element(bytes: &[u8]) -> Option<(u8, &[u8], &[u8])> {
    let tag = *bytes.first()?;
    let first = *bytes.get(1)? as usize;
    let (length, header) = if first & 0x80 == 0 {
        (first, 2)
    } else {
        let width = first & 0x7F;
        if width == 0 || width > 4 {
            return None;
        }
        let mut length = 0usize;
        for at in 0..width {
            length = (length << 8) | *bytes.get(2 + at)? as usize;
        }
        (length, 2 + width)
    };
    let end = header.checked_add(length)?;
    if end > bytes.len() {
        return None;
    }
    Some((tag, &bytes[header..end], &bytes[end..]))
}

/// The elements a SEQUENCE's contents hold, in order.
fn elements(mut bytes: &[u8]) -> Option<Vec<(u8, &[u8])>> {
    let mut found = Vec::new();
    while !bytes.is_empty() {
        let (tag, contents, rest) = element(bytes)?;
        found.push((tag, contents));
        bytes = rest;
    }
    Some(found)
}

const SEQUENCE: u8 = 0x30;
const BIT_STRING: u8 = 0x03;
const INTEGER: u8 = 0x02;
const EXPLICIT_VERSION: u8 = 0xA0;

/// The RSA public key a certificate's subjectPublicKeyInfo holds.
pub fn rsa_public_key(der: &[u8]) -> Option<RsaKey> {
    let (tag, certificate, _) = element(der)?;
    if tag != SEQUENCE {
        return None;
    }
    let parts = elements(certificate)?;
    let (tag, tbs) = *parts.first()?;
    if tag != SEQUENCE {
        return None;
    }
    let fields = elements(tbs)?;
    // The version is written in front only when it is not the first.
    let skip = usize::from(fields.first()?.0 == EXPLICIT_VERSION);
    let (tag, key_info) = *fields.get(skip + 5)?;
    if tag != SEQUENCE {
        return None;
    }
    let info = elements(key_info)?;
    let (tag, bits) = *info.get(1)?;
    if tag != BIT_STRING || bits.first() != Some(&0) {
        return None;
    }
    let (tag, key, _) = element(&bits[1..])?;
    if tag != SEQUENCE {
        return None;
    }
    let numbers = elements(key)?;
    let (modulus_tag, modulus) = *numbers.first()?;
    let (exponent_tag, exponent) = *numbers.get(1)?;
    if modulus_tag != INTEGER || exponent_tag != INTEGER {
        return None;
    }
    Some(RsaKey {
        modulus: BigUint::from_bytes_be(modulus),
        public_exponent: BigUint::from_bytes_be(exponent),
        private_exponent: None,
    })
}
