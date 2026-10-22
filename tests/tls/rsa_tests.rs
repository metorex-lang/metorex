use super::bytes;
use super::key_vectors::*;
use metorex::tls::certificate::rsa_public_key;
use metorex::tls::crypto::{RsaKey, SHA256_DIGEST_INFO};
use num_bigint::BigUint;

fn number(hex: &str) -> BigUint {
    BigUint::from_bytes_be(&bytes(hex))
}

fn private_key() -> RsaKey {
    RsaKey {
        modulus: number(MODULUS),
        public_exponent: number(PUBLIC_EXPONENT),
        private_exponent: Some(number(PRIVATE_EXPONENT)),
    }
}

#[test]
fn a_pss_signature_openssl_made_verifies() {
    assert!(private_key().verify_pss(SIGNED_MESSAGE, &bytes(PSS_SIGNATURE)));
}

#[test]
fn a_pss_signature_over_another_message_does_not_verify() {
    assert!(!private_key().verify_pss(b"another message", &bytes(PSS_SIGNATURE)));
}

#[test]
fn a_pss_signature_made_here_verifies() {
    let key = private_key();
    let signature = key.sign_pss(SIGNED_MESSAGE, &[7; 32]).unwrap();

    assert!(key.verify_pss(SIGNED_MESSAGE, &signature));
}

#[test]
fn a_pkcs1_signature_made_here_is_the_one_openssl_made() {
    let digest = bytes(SIGNED_MESSAGE_SHA256);

    assert_eq!(
        private_key().sign_pkcs1(&SHA256_DIGEST_INFO, &digest),
        Some(bytes(PKCS1_SIGNATURE))
    );
}

#[test]
fn a_certificate_gives_the_public_key_it_holds() {
    let key = rsa_public_key(&bytes(CERTIFICATE)).unwrap();

    assert_eq!(
        (key.modulus, key.public_exponent),
        (number(MODULUS), number(PUBLIC_EXPONENT))
    );
}
