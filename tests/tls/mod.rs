mod chacha20_poly1305_tests;
mod connection_tests;
mod gcm_tests;
mod key_derivation_tests;
mod key_vectors;
mod resumption_tests;
mod rsa_tests;
mod tls12_connection_tests;
mod x25519_tests;

fn bytes(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|at| u8::from_str_radix(&hex[at..at + 2], 16).unwrap())
        .collect()
}

fn array(hex: &str) -> [u8; 32] {
    bytes(hex).try_into().unwrap()
}
