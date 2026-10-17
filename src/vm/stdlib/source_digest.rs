//! A digest of a library's source, which names the parsed copy the build
//! script made of it. The build script reads this file too.

const OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const PRIME: u64 = 0x0000_0100_0000_01b3;

/// The 64-bit FNV-1a digest of `source`.
pub(crate) fn source_digest(source: &str) -> u64 {
    source.bytes().fold(OFFSET_BASIS, |digest, byte| {
        (digest ^ u64::from(byte)).wrapping_mul(PRIME)
    })
}
