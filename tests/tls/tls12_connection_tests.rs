use super::bytes;
use super::key_vectors::*;
use metorex::tls::Connection;
use metorex::tls::connection::{TLS1_2, TLS1_3};
use metorex::tls::crypto::RsaKey;
use num_bigint::BigUint;

fn key() -> RsaKey {
    RsaKey {
        modulus: BigUint::from_bytes_be(&bytes(MODULUS)),
        public_exponent: BigUint::from_bytes_be(&bytes(PUBLIC_EXPONENT)),
        private_exponent: Some(BigUint::from_bytes_be(&bytes(PRIVATE_EXPONENT))),
    }
}

fn held(mut connection: Connection, min: u16, max: u16) -> Connection {
    connection.limit_versions(min, max);
    connection
}

fn server() -> Connection {
    Connection::server(vec![bytes(CERTIFICATE)], key())
}

fn tls12_server() -> Connection {
    held(server(), 0, TLS1_2)
}

fn tls12_client() -> Connection {
    held(Connection::client(None), 0, TLS1_2)
}

/// The bytes with the first run of `from` written over with `to`.
fn replaced(written: &[u8], from: &[u8], to: &[u8]) -> Vec<u8> {
    let at = written
        .windows(from.len())
        .position(|window| window == from)
        .expect("the bytes to replace");
    let mut changed = written.to_vec();
    changed[at..at + to.len()].copy_from_slice(to);
    changed
}

/// The first record of `records`, header included.
fn first_record(records: &[u8]) -> Vec<u8> {
    let length = u16::from_be_bytes([records[3], records[4]]) as usize;
    records[..5 + length].to_vec()
}

/// Where the ECDHE parameters of a ServerKeyExchange start in a flight.
fn key_exchange_at(flight: &[u8]) -> usize {
    flight
        .windows(4)
        .position(|window| window == [3, 0, 0x1d, 32])
        .expect("a ServerKeyExchange")
}

/// A client and a server that have shaken hands, each trusting the chain
/// the other sent.
fn shake_hands(mut client: Connection, mut server: Connection) -> (Connection, Connection) {
    let flight = server.feed(&client.start()).unwrap().outgoing;
    client.feed(&flight).unwrap();
    let answer = client.decide_certificate(None).unwrap().outgoing;
    let mut finished = server.feed(&answer).unwrap().outgoing;
    if server.waits_for_certificate_decision() {
        finished = server.decide_certificate(None).unwrap().outgoing;
    }
    client.feed(&finished).unwrap();
    (client, server)
}

/// What a client answers the server's flight with once it trusts the chain.
fn client_answer(client: &mut Connection, server: &mut Connection) -> Vec<u8> {
    let flight = server.feed(&client.start()).unwrap().outgoing;
    client.feed(&flight).unwrap();
    client.decide_certificate(None).unwrap().outgoing
}

/// A ClientHello offering TLS 1.2 with the given suites and no extensions.
fn bare_client_hello(suites: &[u8]) -> Vec<u8> {
    let mut body = vec![3, 3];
    body.extend_from_slice(&[7; 32]);
    body.push(0);
    body.extend_from_slice(&(suites.len() as u16).to_be_bytes());
    body.extend_from_slice(suites);
    body.extend_from_slice(&[1, 0]);
    let mut message = vec![1, 0];
    message.extend_from_slice(&(body.len() as u16).to_be_bytes());
    message.extend(body);
    let mut record = vec![22, 3, 3];
    record.extend_from_slice(&(message.len() as u16).to_be_bytes());
    record.extend(message);
    record
}

fn client_error(flight: &[u8]) -> String {
    let mut client = tls12_client();
    client.start();
    client.feed(flight).unwrap_err().message
}

fn server_error(hello: &[u8]) -> String {
    tls12_server().feed(hello).unwrap_err().message
}

#[test]
fn a_server_held_to_tls_1_2_agrees_to_it() {
    let (client, server) = shake_hands(Connection::client(None), tls12_server());

    assert_eq!(
        (
            client.is_established(),
            server.is_established(),
            client.version(),
            server.version()
        ),
        (true, true, TLS1_2, TLS1_2)
    );
}

#[test]
fn a_client_held_to_tls_1_2_agrees_to_it() {
    let (client, server) = shake_hands(tls12_client(), server());

    assert_eq!((client.version(), server.is_established()), (TLS1_2, true));
}

#[test]
fn data_the_client_writes_reaches_the_server() {
    let (mut client, mut server) = shake_hands(tls12_client(), tls12_server());
    let records = client.write(b"a001 CAPABILITY\r\n").unwrap();

    assert_eq!(
        server.feed(&records).unwrap().plaintext,
        b"a001 CAPABILITY\r\n"
    );
}

#[test]
fn data_the_server_writes_reaches_the_client_across_records() {
    let (mut client, mut server) = shake_hands(tls12_client(), tls12_server());
    let records = server.write(&[b'x'; 40000]).unwrap();

    assert_eq!(client.feed(&records).unwrap().plaintext, vec![b'x'; 40000]);
}

#[test]
fn records_fed_a_byte_at_a_time_are_read_once_whole() {
    let (mut client, mut server) = shake_hands(tls12_client(), tls12_server());
    let records = client.write(b"220 ready").unwrap();
    let mut read = Vec::new();
    for byte in records {
        read.extend(server.feed(&[byte]).unwrap().plaintext);
    }

    assert_eq!(read, b"220 ready");
}

#[test]
fn a_changed_record_fails_its_check() {
    let (mut client, mut server) = shake_hands(tls12_client(), tls12_server());
    let mut records = client.write(b"hello").unwrap();
    records[20] ^= 1;

    assert_eq!(server.feed(&records).unwrap_err().message, "bad record mac");
}

#[test]
fn a_record_too_short_for_its_nonce_and_tag_fails_its_check() {
    let (_, mut server) = shake_hands(tls12_client(), tls12_server());

    assert_eq!(
        server
            .feed(&[23, 3, 3, 0, 4, 0, 0, 0, 0])
            .unwrap_err()
            .message,
        "bad record mac"
    );
}

#[test]
fn a_server_asking_for_a_certificate_holds_the_one_the_client_sends() {
    let client = tls12_client().with_client_certificate(vec![bytes(CERTIFICATE)], key());
    let (client, server) = shake_hands(client, tls12_server().requesting_certificate(true));

    assert_eq!(
        (client.is_established(), server.peer_certificates.clone()),
        (true, vec![bytes(CERTIFICATE)])
    );
}

#[test]
fn a_server_asking_for_a_certificate_goes_on_without_one_when_not_required() {
    let (client, server) =
        shake_hands(tls12_client(), tls12_server().requesting_certificate(false));

    assert!(client.is_established() && server.is_established());
}

#[test]
fn a_server_requiring_a_certificate_refuses_a_client_with_none() {
    let mut client = tls12_client();
    let mut server = tls12_server().requesting_certificate(true);
    let answer = client_answer(&mut client, &mut server);

    assert_eq!(
        server.feed(&answer).unwrap_err().message,
        "peer did not return a certificate"
    );
}

#[test]
fn a_server_refuses_a_client_certificate_proof_that_was_changed() {
    let mut client = tls12_client().with_client_certificate(vec![bytes(CERTIFICATE)], key());
    let mut server = tls12_server().requesting_certificate(true);
    let mut answer = client_answer(&mut client, &mut server);
    let end = first_record(&answer).len();
    answer[end - 1] ^= 1;

    assert_eq!(server.feed(&answer).unwrap_err().message, "bad signature");
}

#[test]
fn a_client_refuses_a_server_with_no_certificate() {
    let mut server = held(Connection::server(Vec::new(), key()), 0, TLS1_2);
    let mut client = tls12_client();
    let flight = server.feed(&client.start()).unwrap().outgoing;

    assert_eq!(
        client.feed(&flight).unwrap_err().message,
        "peer did not return a certificate"
    );
}

#[test]
fn a_client_refuses_a_certificate_with_no_rsa_key() {
    let mut server = held(Connection::server(vec![vec![0x30, 0]], key()), 0, TLS1_2);
    let mut client = tls12_client();
    let flight = server.feed(&client.start()).unwrap().outgoing;

    assert_eq!(
        client.feed(&flight).unwrap_err().message,
        "unsupported certificate key"
    );
}

#[test]
fn a_client_refuses_a_changed_key_exchange_signature() {
    let mut server = tls12_server();
    let mut client = tls12_client();
    let mut flight = server.feed(&client.start()).unwrap().outgoing;
    // ServerHelloDone, four bytes, follows the signature.
    let last = flight.len() - 5;
    flight[last] ^= 1;

    assert_eq!(client.feed(&flight).unwrap_err().message, "bad signature");
}

#[test]
fn a_client_refuses_a_key_exchange_signed_with_an_unknown_scheme() {
    let mut server = tls12_server();
    let mut client = tls12_client();
    let mut flight = server.feed(&client.start()).unwrap().outgoing;
    let scheme = key_exchange_at(&flight) + 36;
    flight[scheme..scheme + 2].copy_from_slice(&[8, 5]);

    assert_eq!(client.feed(&flight).unwrap_err().message, "bad signature");
}

#[test]
fn a_client_refuses_a_key_exchange_on_another_curve() {
    let mut server = tls12_server();
    let mut client = tls12_client();
    let mut flight = server.feed(&client.start()).unwrap().outgoing;
    let group = key_exchange_at(&flight) + 2;
    flight[group] = 0x17;

    assert_eq!(client.feed(&flight).unwrap_err().message, "wrong curve");
}

#[test]
fn a_client_refuses_a_tls_1_2_hello_with_the_tls_1_3_suite() {
    let mut server = tls12_server();
    let mut client = tls12_client();
    let mut flight = server.feed(&client.start()).unwrap().outgoing;
    // The record and message headers, the version, the random, and an
    // empty session id come before the suite.
    flight[44..46].copy_from_slice(&[0x13, 0x01]);

    assert_eq!(
        client.feed(&flight).unwrap_err().message,
        "no shared cipher"
    );
}

#[test]
fn a_client_offering_tls_1_3_refuses_a_server_that_marks_a_downgrade() {
    let mut server = server();
    let flight = server.feed(&tls12_client().start()).unwrap().outgoing;
    let mut client = Connection::client(None);
    client.start();

    assert_eq!(
        client.feed(&flight).unwrap_err().message,
        "inappropriate fallback"
    );
}

#[test]
fn a_client_held_to_tls_1_3_refuses_a_tls_1_2_hello() {
    let mut server = server();
    let flight = server.feed(&tls12_client().start()).unwrap().outgoing;
    let mut client = held(Connection::client(None), TLS1_3, 0);
    client.start();

    assert_eq!(
        client.feed(&flight).unwrap_err().message,
        "unsupported protocol"
    );
}

#[test]
fn a_server_held_to_tls_1_2_refuses_a_client_held_to_tls_1_3() {
    let mut client = held(Connection::client(None), TLS1_3, 0);

    assert_eq!(server_error(&client.start()), "unsupported protocol");
}

#[test]
fn a_server_held_to_tls_1_3_refuses_a_client_held_to_tls_1_2() {
    let mut server = held(server(), TLS1_3, 0);

    assert_eq!(
        server.feed(&tls12_client().start()).unwrap_err().message,
        "unsupported protocol"
    );
}

#[test]
fn a_client_checks_a_key_exchange_signed_with_pkcs1() {
    let mut client = tls12_client();
    let hello = replaced(&client.start(), &[8, 4, 4, 1], &[4, 1, 4, 1]);
    let flight = tls12_server().feed(&hello).unwrap().outgoing;
    client.feed(&flight).unwrap();

    assert!(client.waits_for_certificate_decision());
}

#[test]
fn a_server_refuses_a_finished_over_a_hello_it_did_not_read() {
    let mut client = tls12_client();
    let mut server = tls12_server();
    let hello = replaced(&client.start(), &[8, 4, 4, 1], &[4, 1, 4, 1]);
    let flight = server.feed(&hello).unwrap().outgoing;
    client.feed(&flight).unwrap();
    let answer = client.decide_certificate(None).unwrap().outgoing;

    assert_eq!(
        server.feed(&answer).unwrap_err().message,
        "digest check failed"
    );
}

#[test]
fn a_server_refuses_a_client_with_no_rsa_signature_scheme() {
    let hello = replaced(&tls12_client().start(), &[8, 4, 4, 1], &[4, 3, 5, 3]);

    assert_eq!(server_error(&hello), "no suitable signature algorithm");
}

#[test]
fn a_server_refuses_a_client_without_x25519() {
    let hello = replaced(
        &tls12_client().start(),
        &[0, 0x0a, 0, 4, 0, 2, 0, 0x1d],
        &[0, 0x0a, 0, 4, 0, 2, 0, 0x17],
    );

    assert_eq!(server_error(&hello), "no shared cipher");
}

#[test]
fn a_server_refuses_a_client_without_the_tls_1_2_suite() {
    let hello = replaced(
        &tls12_client().start(),
        &[0, 2, 0xc0, 0x2f],
        &[0, 2, 0, 0x9c],
    );

    assert_eq!(server_error(&hello), "no shared cipher");
}

#[test]
fn a_hello_with_no_extensions_gets_an_answer_with_none() {
    let flight = tls12_server()
        .feed(&bare_client_hello(&[0xc0, 0x2f]))
        .unwrap()
        .outgoing;

    // The answer signs over another client's random.
    assert_eq!(client_error(&flight), "bad signature");
}

#[test]
fn a_server_answers_the_renegotiation_suite_with_the_extension() {
    let flight = tls12_server()
        .feed(&bare_client_hello(&[0xc0, 0x2f, 0, 0xff]))
        .unwrap()
        .outgoing;

    assert!(flight.windows(5).any(|window| window == [0xff, 1, 0, 1, 0]));
}

#[test]
fn a_server_refuses_a_key_exchange_point_of_the_wrong_length() {
    let mut server = tls12_server();
    server.feed(&tls12_client().start()).unwrap();
    let mut exchange = vec![22, 3, 3, 0, 36, 16, 0, 0, 32, 31];
    exchange.extend_from_slice(&[9; 31]);

    assert_eq!(server.feed(&exchange).unwrap_err().message, "bad ecpoint");
}

#[test]
fn a_server_refuses_a_finished_sent_before_change_cipher_spec() {
    let mut client = tls12_client();
    let mut server = tls12_server();
    let answer = client_answer(&mut client, &mut server);
    server.feed(&first_record(&answer)).unwrap();
    let mut finished = vec![22, 3, 3, 0, 16, 20, 0, 0, 12];
    finished.extend_from_slice(&[0; 12]);

    assert_eq!(
        server.feed(&finished).unwrap_err().message,
        "unexpected message"
    );
}

#[test]
fn a_server_reads_a_sealed_finished_without_change_cipher_spec_as_unexpected() {
    let mut client = tls12_client();
    let mut server = tls12_server();
    let answer = client_answer(&mut client, &mut server);
    let handshake = first_record(&answer);
    // ChangeCipherSpec is a six-byte record after the handshake record.
    let mut skipped = handshake.clone();
    skipped.extend_from_slice(&answer[handshake.len() + 6..]);

    assert_eq!(
        server.feed(&skipped).unwrap_err().message,
        "unexpected message"
    );
}
