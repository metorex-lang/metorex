// RFC 5869's first test case, the TLS 1.3 key schedule of RFC 8448's
// simple handshake, and the TLS 1.2 PRF test vector for SHA-256 that the
// IETF TLS working group published.

use super::bytes;
use metorex::tls::crypto::{derive_secret, expand_label, hkdf_expand, hkdf_extract, prf};

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

#[test]
fn the_tls_1_2_prf_gives_the_published_output() {
    assert_eq!(
        prf(
            &bytes("9bbe436ba940f017b17652849a71db35"),
            "test label",
            &bytes("a0ba9f936cda311827a6f796ffd5198c"),
            100
        ),
        bytes(concat!(
            "e3f229ba727be17b8d122620557cd453c2aab21d07c3d495329b52d4e61edb5a",
            "6b301791e90d35c9c9a46b4e14baf9af0fa022f7077def17abfd3797c0564bab",
            "4fbc91666e9def9b97fce34f796789baa48082d122ee42c5a72e5a5110fff701",
            "87347b66"
        ))
    );
}
