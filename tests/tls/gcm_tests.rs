// Test cases 1, 2, and 4 of the GCM specification, the last checked
// against MRI's OpenSSL.

use super::bytes;
use metorex::tls::gcm::Gcm;

fn nonce(hex: &str) -> [u8; 12] {
    bytes(hex).try_into().unwrap()
}

#[test]
fn sealing_nothing_under_a_zero_key_gives_only_the_tag() {
    let gcm = Gcm::new(&[0; 16]).unwrap();

    assert_eq!(
        gcm.seal(&[0; 12], &[], &[]),
        bytes("58e2fccefa7e3061367f1d57a4e7455a")
    );
}

#[test]
fn sealing_a_zero_block_under_a_zero_key_gives_the_listed_text_and_tag() {
    let gcm = Gcm::new(&[0; 16]).unwrap();

    assert_eq!(
        gcm.seal(&[0; 12], &[], &[0; 16]),
        bytes("0388dace60b6a392f328c2b971b2fe78ab6e47d42cec13bdf53a67b21257bddf")
    );
}

const KEY: &str = "feffe9928665731c6d6a8f9467308308";
const NONCE: &str = "cafebabefacedbaddecaf888";
const ADDITIONAL: &str = "feedfacedeadbeeffeedfacedeadbeefabaddad2";
const PLAINTEXT: &str = concat!(
    "d9313225f88406e5a55909c5aff5269a86a7a9531534f7da2e4c303d8a318a72",
    "1c3c0c95956809532fcf0e2449a6b525b16aedf5aa0de657ba637b39"
);
const SEALED: &str = concat!(
    "42831ec2217774244b7221b784d0d49ce3aa212f2c02a4e035c17e2329aca12e",
    "21d514b25466931c7d8f6a5aac84aa051ba30b396a0aac973d58e091",
    "5bc94fbc3221a5db94fae95ae7121a47"
);

#[test]
fn sealing_a_partial_block_with_additional_data_gives_the_listed_text_and_tag() {
    let gcm = Gcm::new(&bytes(KEY)).unwrap();

    assert_eq!(
        gcm.seal(&nonce(NONCE), &bytes(ADDITIONAL), &bytes(PLAINTEXT)),
        bytes(SEALED)
    );
}

#[test]
fn opening_the_sealed_text_gives_the_plaintext_back() {
    let gcm = Gcm::new(&bytes(KEY)).unwrap();

    assert_eq!(
        gcm.open(&nonce(NONCE), &bytes(ADDITIONAL), &bytes(SEALED)),
        Some(bytes(PLAINTEXT))
    );
}

#[test]
fn opening_text_whose_additional_data_changed_fails() {
    let gcm = Gcm::new(&bytes(KEY)).unwrap();

    assert_eq!(gcm.open(&nonce(NONCE), b"other", &bytes(SEALED)), None);
}
