// The test vectors of RFC 7748, section 5.2 and section 6.1.

use super::array;
use metorex::tls::x25519::{public_key, x25519};

#[test]
fn a_scalar_times_a_point_gives_the_rfc_7748_result() {
    let scalar = array("a546e36bf0527c9d3b16154b82465edd62144c0ac1fc5a18506a2244ba449ac4");
    let point = array("e6db6867583030db3594c1a424b15f7c726624ec26b3353b10a903a6d0ab1c4c");

    assert_eq!(
        x25519(&scalar, &point),
        array("c3da55379de9c6908e94ea4df28d084f32eccf03491c71f754b4075577a28552")
    );
}

#[test]
fn a_private_key_gives_the_public_key_rfc_7748_lists() {
    let alice = array("77076d0a7318a57d3c16c17251b26645df4c2f87ebc0992ab177fba51db92c2a");

    assert_eq!(
        public_key(&alice),
        array("8520f0098930a754748b7ddcb43ef75a0dbf3a0d26381af4eba4a98eaa9b4e6a")
    );
}

#[test]
fn both_sides_agree_on_the_shared_secret_rfc_7748_lists() {
    let alice = array("77076d0a7318a57d3c16c17251b26645df4c2f87ebc0992ab177fba51db92c2a");
    let bob = array("5dab087e624a8a4b79e17f8b83800ee66f3bb1292618b6fd1c2f8b27ff88e0eb");
    let shared = array("4a5d9d5ba4ce2de1728e3bf480350f25e07e21c947d19e3376f09b3c1e161742");

    assert_eq!(
        (
            x25519(&alice, &public_key(&bob)),
            x25519(&bob, &public_key(&alice))
        ),
        (shared, shared)
    );
}
