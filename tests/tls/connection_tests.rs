use super::bytes;
use super::key_vectors::*;
use metorex::tls::crypto::RsaKey;
use metorex::tls::{Connection, Progress};
use num_bigint::BigUint;

fn server() -> Connection {
    let key = RsaKey {
        modulus: BigUint::from_bytes_be(&bytes(MODULUS)),
        public_exponent: BigUint::from_bytes_be(&bytes(PUBLIC_EXPONENT)),
        private_exponent: Some(BigUint::from_bytes_be(&bytes(PRIVATE_EXPONENT))),
    };
    Connection::server(vec![bytes(CERTIFICATE)], key)
}

/// A client and a server that have shaken hands, the client trusting the
/// server's chain.
fn established() -> (Connection, Connection) {
    let mut client = Connection::client(Some("localhost".to_string()));
    let mut server = server();
    let hello = client.start();
    let flight = server.feed(&hello).unwrap().outgoing;
    client.feed(&flight).unwrap();
    let finished = client.decide_certificate(None).unwrap().outgoing;
    server.feed(&finished).unwrap();
    (client, server)
}

#[test]
fn a_client_pauses_for_the_chain_decision_after_the_server_flight() {
    let mut client = Connection::client(Some("localhost".to_string()));
    let mut server = server();
    let flight = server.feed(&client.start()).unwrap().outgoing;
    client.feed(&flight).unwrap();

    assert!(client.waits_for_certificate_decision());
}

#[test]
fn the_client_holds_the_chain_the_server_sent() {
    let mut client = Connection::client(None);
    let mut server = server();
    let flight = server.feed(&client.start()).unwrap().outgoing;
    client.feed(&flight).unwrap();

    assert_eq!(client.peer_certificates, vec![bytes(CERTIFICATE)]);
}

#[test]
fn the_server_reads_the_name_the_client_asked_for() {
    let mut client = Connection::client(Some("localhost".to_string()));
    let mut server = server();
    server.feed(&client.start()).unwrap();

    assert_eq!(server.requested_name.as_deref(), Some("localhost"));
}

#[test]
fn both_sides_finish_the_handshake() {
    let (client, server) = established();

    assert_eq!(
        (client.is_established(), server.is_established()),
        (true, true)
    );
}

#[test]
fn data_the_client_writes_reaches_the_server() {
    let (mut client, mut server) = established();
    let records = client.write(b"a001 CAPABILITY\r\n").unwrap();

    assert_eq!(
        server.feed(&records).unwrap().plaintext,
        b"a001 CAPABILITY\r\n"
    );
}

#[test]
fn data_the_server_writes_reaches_the_client() {
    let (mut client, mut server) = established();
    let records = server.write(&[b'x'; 40000]).unwrap();

    assert_eq!(client.feed(&records).unwrap().plaintext, vec![b'x'; 40000]);
}

#[test]
fn records_fed_a_byte_at_a_time_are_read_once_whole() {
    let (mut client, mut server) = established();
    let records = client.write(b"220 ready").unwrap();
    let mut read = Vec::new();
    for byte in records {
        read.extend(server.feed(&[byte]).unwrap().plaintext);
    }

    assert_eq!(read, b"220 ready");
}

#[test]
fn a_close_reaches_the_peer_as_a_close() {
    let (mut client, mut server) = established();
    let close = client.close();
    let Progress { peer_closed, .. } = server.feed(&close).unwrap();

    assert!(peer_closed);
}

#[test]
fn a_refused_chain_sends_the_server_an_alert_it_reports() {
    let mut client = Connection::client(None);
    let mut server = server();
    let flight = server.feed(&client.start()).unwrap().outgoing;
    client.feed(&flight).unwrap();
    let alert = client.decide_certificate(Some(48)).unwrap().outgoing;

    assert_eq!(
        server.feed(&alert).unwrap_err().message,
        "tlsv1 alert unknown ca (SSL alert number 48)"
    );
}

#[test]
fn a_changed_record_fails_its_check() {
    let (mut client, mut server) = established();
    let mut records = client.write(b"hello").unwrap();
    records[7] ^= 1;

    assert_eq!(server.feed(&records).unwrap_err().message, "bad record mac");
}
