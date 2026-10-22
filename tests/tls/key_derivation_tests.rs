// RFC 5869's first test case, and the TLS 1.3 key schedule of RFC 8448's
// simple handshake.

use super::bytes;
use metorex::tls::crypto::{derive_secret, expand_label, hkdf_expand, hkdf_extract};

#[test]
fn hkdf_gives_the_keying_material_rfc_5869_lists() {
    let pseudorandom = hkdf_extract(&bytes("000102030405060708090a0b0c"), &[0x0b; 22]);

    assert_eq!(
        hkdf_expand(&pseudorandom, &bytes("f0f1f2f3f4f5f6f7f8f9"), 42),
        bytes(
            "3cb25f25faacd57a90434f64d0362f2a2d2d0a90cf1a5a4c5db02d56ecc4c5bf34007208d5b887185865"
        )
    );
}

#[test]
fn the_early_secret_with_no_preshared_key_is_the_one_rfc_8448_lists() {
    assert_eq!(
        hkdf_extract(&[], &[0; 32]),
        bytes("33ad0a1c607ec03b09e6cd9893680ce210adf300aa1f2660e1b22e10f170f92a")
    );
}

#[test]
fn the_derived_secret_of_the_early_secret_is_the_one_rfc_8448_lists() {
    let early = hkdf_extract(&[], &[0; 32]);

    assert_eq!(
        derive_secret(&early, "derived", &[]),
        bytes("6f2615a108c702c5678f54fc9dbab69716c076189c48250cebeac3576c3611ba")
    );
}

#[test]
fn a_label_expanded_to_a_key_has_the_length_asked_for() {
    assert_eq!(expand_label(&[1; 32], "key", &[], 16).len(), 16);
}
