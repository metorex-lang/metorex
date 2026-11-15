// Sealed bytes checked against MRI's OpenSSL.

use super::bytes;
use metorex::tls::chacha20_poly1305::ChaCha20Poly1305;

fn counting_key() -> ChaCha20Poly1305 {
    ChaCha20Poly1305::new(&(0..32).collect::<Vec<u8>>()).unwrap()
}

const COUNTING_NONCE: [u8; 12] = [100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111];

const SEALED_HEADER_AND_150_BYTES: &str = "4c69c3b2e0dd0e2a83a66d078af1a0eb08beea4ce72893d5be478eca824ab48bb2c1ca98d742320be797b29590db90b0bca3f76567134e9126c793ee7cce29896a344d1ce7cd1810933e14c3aca76bb7196ae06ddf054d54d9f305921581776fc03066d780091c6447fd5f11e3f9e7575970db4163913b63b728449ee9df1fdacca3de56b5cc81fbe811908f5b02bd4e854b477e7e8bd666292a71a912db65874c51ea26133b";

#[test]
fn sealing_nothing_gives_only_the_tag() {
    assert_eq!(
        counting_key().seal(&COUNTING_NONCE, &[], &[]),
        bytes("a33126986fcb1312c2476c125059ffe4")
    );
}

#[test]
fn sealing_text_longer_than_two_blocks_with_additional_data() {
    assert_eq!(
        counting_key().seal(&COUNTING_NONCE, b"header", &[b'x'; 150]),
        bytes(SEALED_HEADER_AND_150_BYTES)
    );
}

#[test]
fn sealing_under_an_all_ones_key_carries_through_every_limb() {
    let cipher = ChaCha20Poly1305::new(&[0xff; 32]).unwrap();

    assert_eq!(
        cipher.seal(&[0xff; 12], &[0xff; 13], &[0xff; 64]),
        bytes(
            "22d8bfc313e141bd692c82cec7785b295f8ddae08dbbc49a2deb545ef8a39534d8c0bdab2521070568ade0405d54eebcfecd35122cb5133d931f64722e053522a8c99924748713548362473b2095e71e"
        )
    );
}

#[test]
fn opening_what_was_sealed_gives_the_plaintext_back() {
    let sealed = bytes(SEALED_HEADER_AND_150_BYTES);

    assert_eq!(
        counting_key().open(&COUNTING_NONCE, b"header", &sealed),
        Some(vec![b'x'; 150])
    );
}

#[test]
fn opening_with_a_changed_tag_gives_nothing() {
    let mut sealed = bytes(SEALED_HEADER_AND_150_BYTES);
    *sealed.last_mut().unwrap() ^= 1;

    assert_eq!(
        counting_key().open(&COUNTING_NONCE, b"header", &sealed),
        None
    );
}

#[test]
fn opening_fewer_bytes_than_a_tag_gives_nothing() {
    assert_eq!(counting_key().open(&COUNTING_NONCE, &[], &[0; 15]), None);
}

#[test]
fn a_key_of_any_length_but_32_bytes_is_refused() {
    assert!(ChaCha20Poly1305::new(&[0; 16]).is_none());
}
