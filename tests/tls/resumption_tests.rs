use super::bytes;
use super::key_vectors::*;
use metorex::tls::Connection;
use metorex::tls::connection::{TLS1_2, TLS1_3};
use metorex::tls::crypto::RsaKey;
use metorex::tls::session::Session;
use num_bigint::BigUint;

fn server(version: u16) -> Connection {
    let key = RsaKey {
        modulus: BigUint::from_bytes_be(&bytes(MODULUS)),
        public_exponent: BigUint::from_bytes_be(&bytes(PUBLIC_EXPONENT)),
        private_exponent: Some(BigUint::from_bytes_be(&bytes(PRIVATE_EXPONENT))),
    };
    let mut server = Connection::server(vec![bytes(CERTIFICATE)], key);
    server.limit_versions(version, version);
    server
}

fn client(version: u16, session: Option<Session>) -> Connection {
    let mut client = Connection::client(None);
    client.limit_versions(version, version);
    if let Some(session) = session {
        client.offer_session(session);
    }
    client
}

/// Shake hands, the client trusting the server's chain when asked, and
/// hand the client what the server sends after the client's Finished.
fn shake_hands(mut client: Connection, mut server: Connection) -> (Connection, Connection) {
    let hello = client.start();
    let flight = server.feed(&hello).unwrap().outgoing;
    let mut answer = client.feed(&flight).unwrap();
    if client.waits_for_certificate_decision() {
        answer = client.decide_certificate(None).unwrap();
    }
    let after = server.feed(&answer.outgoing).unwrap().outgoing;
    client.feed(&after).unwrap();
    (client, server)
}

/// The sessions a full handshake at `version` gives the client.
fn first_sessions(version: u16) -> Vec<Session> {
    let (mut client, _) = shake_hands(client(version, None), server(version));
    client.take_new_sessions()
}

fn resumed_pair(version: u16) -> (Connection, Connection) {
    let session = first_sessions(version).remove(0);
    shake_hands(client(version, Some(session)), server(version))
}

fn a_session() -> Session {
    Session {
        version: TLS1_3,
        suite: 0x1301,
        id: vec![1; 32],
        secret: vec![2; 32],
        ticket: vec![3; 40],
        age_add: 7,
        issued_at: 1_700_000_000_000,
        lifetime: 7200,
        peer_certificates: vec![vec![4; 5], vec![5; 3]],
    }
}

#[test]
fn a_session_reads_back_from_its_bytes() {
    let session = a_session();

    assert_eq!(Session::from_bytes(&session.to_bytes()), Some(session));
}

#[test]
fn bytes_cut_short_hold_no_session() {
    let written = a_session().to_bytes();

    assert_eq!(Session::from_bytes(&written[..written.len() - 1]), None);
}

#[test]
fn bytes_with_more_after_the_session_hold_no_session() {
    let mut written = a_session().to_bytes();
    written.push(0);

    assert_eq!(Session::from_bytes(&written), None);
}

#[test]
fn a_tls_1_2_client_is_given_one_session_with_a_ticket() {
    let sessions = first_sessions(TLS1_2);

    assert_eq!(
        (
            sessions.len(),
            sessions[0].version,
            sessions[0].ticket.is_empty()
        ),
        (1, TLS1_2, false)
    );
}

#[test]
fn a_tls_1_3_client_is_given_two_tickets() {
    assert_eq!(first_sessions(TLS1_3).len(), 2);
}

#[test]
fn a_full_handshake_is_not_resumed() {
    let (client, server) = shake_hands(client(TLS1_3, None), server(TLS1_3));

    assert_eq!((client.is_resumed(), server.is_resumed()), (false, false));
}

#[test]
fn both_sides_resume_a_tls_1_2_session() {
    let (client, server) = resumed_pair(TLS1_2);

    assert_eq!(
        (
            client.is_established(),
            client.is_resumed(),
            server.is_resumed()
        ),
        (true, true, true)
    );
}

#[test]
fn both_sides_resume_a_tls_1_3_session() {
    let (client, server) = resumed_pair(TLS1_3);

    assert_eq!(
        (
            client.is_established(),
            client.is_resumed(),
            server.is_resumed()
        ),
        (true, true, true)
    );
}

#[test]
fn a_resumed_tls_1_2_client_is_given_no_new_session() {
    let (mut client, _) = resumed_pair(TLS1_2);

    assert_eq!(client.take_new_sessions().len(), 0);
}

#[test]
fn a_resumed_tls_1_3_client_is_given_one_ticket() {
    let (mut client, _) = resumed_pair(TLS1_3);

    assert_eq!(client.take_new_sessions().len(), 1);
}

#[test]
fn a_resumed_client_holds_the_chain_of_the_first_handshake() {
    let (client, _) = resumed_pair(TLS1_3);

    assert_eq!(client.peer_certificates, vec![bytes(CERTIFICATE)]);
}

#[test]
fn a_resumed_server_holds_the_session_it_resumed() {
    let (client, server) = resumed_pair(TLS1_2);

    assert_eq!(
        server.session().map(|session| session.secret.clone()),
        client.session().map(|session| session.secret.clone())
    );
}

#[test]
fn data_crosses_a_resumed_tls_1_2_connection() {
    let (mut client, mut server) = resumed_pair(TLS1_2);
    let records = client.write(b"GET / HTTP/1.1\r\n").unwrap();

    assert_eq!(
        server.feed(&records).unwrap().plaintext,
        b"GET / HTTP/1.1\r\n"
    );
}

#[test]
fn data_crosses_a_resumed_tls_1_3_connection() {
    let (mut client, mut server) = resumed_pair(TLS1_3);
    let records = server.write(b"HTTP/1.1 200 OK\r\n").unwrap();

    assert_eq!(
        client.feed(&records).unwrap().plaintext,
        b"HTTP/1.1 200 OK\r\n"
    );
}

#[test]
fn a_ticket_the_server_did_not_seal_gets_a_full_tls_1_2_handshake() {
    let mut session = first_sessions(TLS1_2).remove(0);
    session.ticket = vec![9; 3];
    let (client, server) = shake_hands(client(TLS1_2, Some(session)), server(TLS1_2));

    assert_eq!(
        (
            client.is_established(),
            client.is_resumed(),
            server.is_resumed()
        ),
        (true, false, false)
    );
}

#[test]
fn a_ticket_the_server_did_not_seal_gets_a_full_tls_1_3_handshake() {
    let (client, server) = shake_hands(client(TLS1_3, Some(a_session())), server(TLS1_3));

    assert_eq!(
        (
            client.is_established(),
            client.is_resumed(),
            server.is_resumed()
        ),
        (true, false, false)
    );
}

#[test]
fn a_tls_1_3_session_is_not_offered_over_tls_1_2() {
    let session = first_sessions(TLS1_3).remove(0);
    let (client, _) = shake_hands(client(TLS1_2, Some(session)), server(TLS1_2));

    assert_eq!(
        (client.is_established(), client.is_resumed()),
        (true, false)
    );
}

#[test]
fn a_session_whose_suite_hash_the_client_no_longer_allows_is_not_offered() {
    let session = first_sessions(TLS1_3).remove(0);
    let other = if session.suite == 0x1302 {
        "TLS_AES_128_GCM_SHA256"
    } else {
        "TLS_AES_256_GCM_SHA384"
    };
    let mut held = client(TLS1_3, Some(session));
    held.limit_suites(&[other.to_string()]);
    let (client, _) = shake_hands(held, server(TLS1_3));

    assert_eq!(
        (client.is_established(), client.is_resumed()),
        (true, false)
    );
}

#[test]
fn a_server_choosing_a_suite_of_another_hash_does_not_resume() {
    let session = first_sessions(TLS1_3).remove(0);
    let mut held = server(TLS1_3);
    let other = if session.suite == 0x1302 {
        "TLS_AES_128_GCM_SHA256"
    } else {
        "TLS_AES_256_GCM_SHA384"
    };
    held.limit_suites(&[other.to_string()]);
    let (client, server) = shake_hands(client(TLS1_3, Some(session)), held);

    assert_eq!(
        (client.is_established(), server.is_resumed()),
        (true, false)
    );
}

#[test]
fn a_changed_binder_fails_the_handshake() {
    let session = first_sessions(TLS1_3).remove(0);
    let mut hello = client(TLS1_3, Some(session)).start();
    let last = hello.len() - 1;
    hello[last] ^= 1;

    assert_eq!(
        server(TLS1_3).feed(&hello).unwrap_err().message,
        "binder does not verify"
    );
}

/// The psk_key_exchange_modes extension asking for psk_dhe_ke.
const PSK_MODES: [u8; 6] = [0x00, 0x2d, 0x00, 0x02, 0x01, 0x01];

fn without_psk_modes(hello: &[u8]) -> Vec<u8> {
    let at = hello
        .windows(PSK_MODES.len())
        .position(|window| window == PSK_MODES)
        .expect("the psk_key_exchange_modes extension");
    let mut changed = hello.to_vec();
    changed[at..at + 2].copy_from_slice(&[0xff, 0xfe]);
    changed
}

#[test]
fn a_server_does_not_resume_a_client_that_did_not_ask_for_a_key_share() {
    let session = first_sessions(TLS1_3).remove(0);
    let hello = without_psk_modes(&client(TLS1_3, Some(session)).start());
    let mut server = server(TLS1_3);
    server.feed(&hello).unwrap();

    assert!(!server.is_resumed());
}

#[test]
fn a_client_refuses_a_pre_shared_key_it_did_not_offer() {
    let session = first_sessions(TLS1_3).remove(0);
    let mut offering = client(TLS1_3, Some(session));
    let mut server = server(TLS1_3);
    let flight = server.feed(&offering.start()).unwrap().outgoing;
    let mut other = client(TLS1_3, None);
    other.start();

    assert_eq!(other.feed(&flight).unwrap_err().message, "bad psk");
}

#[test]
fn a_client_refuses_a_pre_shared_key_chosen_with_a_suite_of_another_hash() {
    let session = first_sessions(TLS1_3).remove(0);
    let other_suite: u8 = if session.suite == 0x1302 { 0x01 } else { 0x02 };
    let mut offering = client(TLS1_3, Some(session));
    let mut server = server(TLS1_3);
    let mut flight = server.feed(&offering.start()).unwrap().outgoing;
    // Record header, handshake header, version, random, and session id.
    let suite_at = 5 + 4 + 2 + 32 + 1 + 32;
    flight[suite_at + 1] = other_suite;

    assert_eq!(offering.feed(&flight).unwrap_err().message, "bad psk");
}
