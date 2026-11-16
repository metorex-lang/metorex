// Resuming a session: TLS 1.2 session tickets (RFC 5077) and session ids,
// and TLS 1.3 pre-shared keys from NewSessionTicket (RFC 8446, section
// 4.6.1) with a fresh X25519 share.

use super::*;
use std::sync::OnceLock;

/// The key a server seals its tickets with, one for the process.
fn ticket_key() -> &'static ChaCha20Poly1305 {
    static KEY: OnceLock<ChaCha20Poly1305> = OnceLock::new();
    KEY.get_or_init(|| ChaCha20Poly1305::new(&random_bytes::<32>()).expect("a 32-byte key"))
}

const TICKET_NONCE_LENGTH: usize = 12;

impl Connection {
    /// Have a client offer to resume `session`.
    pub fn offer_session(&mut self, session: Session) {
        self.offered_session = Some(session);
    }

    /// Whether the handshake resumed a session.
    pub fn is_resumed(&self) -> bool {
        self.resumed
    }

    /// The session the connection runs in, once established.
    pub fn session(&self) -> Option<&Session> {
        self.session.as_ref()
    }

    /// The sessions a client was given since this was last asked.
    pub fn take_new_sessions(&mut self) -> Vec<Session> {
        std::mem::take(&mut self.new_sessions)
    }

    pub(super) fn seal_ticket(session: &Session) -> Vec<u8> {
        let nonce = random_bytes::<TICKET_NONCE_LENGTH>();
        let mut ticket = nonce.to_vec();
        ticket.extend(ticket_key().seal(&nonce, &[], &session.to_bytes()));
        ticket
    }

    pub(super) fn open_ticket(ticket: &[u8]) -> Option<Session> {
        if ticket.len() < TICKET_NONCE_LENGTH {
            return None;
        }
        let (nonce, sealed) = ticket.split_at(TICKET_NONCE_LENGTH);
        let nonce: [u8; TICKET_NONCE_LENGTH] = nonce.try_into().ok()?;
        Session::from_bytes(&ticket_key().open(&nonce, &[], sealed)?)
    }

    /// The SessionTicket extension's body: the ticket of a TLS 1.2 session
    /// offered, or nothing to ask for a new one.
    pub(super) fn offered_tls12_ticket(&self) -> Vec<u8> {
        match &self.offered_session {
            Some(session) if session.version == TLS1_2 => session.ticket.clone(),
            _ => Vec::new(),
        }
    }

    /// The TLS 1.3 session offered, when its suite's hash is one this side
    /// still allows.
    fn offered_psk_session(&self) -> Option<&Session> {
        self.offered_session.as_ref().filter(|session| {
            session.version == TLS1_3
                && !session.ticket.is_empty()
                && self
                    .allowed_suites
                    .iter()
                    .any(|suite| suite_hash(*suite) == suite_hash(session.suite))
        })
    }

    /// The pre_shared_key extension's body, with a binder of zeros that
    /// `bind_psk` fills in once the rest of the hello is written.
    pub(super) fn psk_offer(&self) -> Option<Vec<u8>> {
        let session = self.offered_psk_session()?;
        let age = now_in_milliseconds().saturating_sub(session.issued_at) as u32;
        let mut identity = with_length(2, &session.ticket);
        identity.extend_from_slice(&age.wrapping_add(session.age_add).to_be_bytes());
        let mut offer = with_length(2, &identity);
        let binder = vec![0u8; suite_hash(session.suite).length()];
        offer.extend(with_length(2, &with_length(1, &binder)));
        Some(offer)
    }

    /// The binder proving the pre-shared key over the hello up to the
    /// binders.
    fn psk_binder(psk: &[u8], hash: crypto::Hash, truncated_hello: &[u8]) -> Vec<u8> {
        let early = crypto::hkdf_extract(hash, &[], psk);
        let binder_key = crypto::derive_secret(hash, &early, "res binder", &[]);
        let key = crypto::expand_label(hash, &binder_key, "finished", &[], hash.length());
        hash.hmac(&key, &hash.digest(truncated_hello))
    }

    pub(super) fn bind_psk(&self, hello: &mut [u8]) {
        let Some(session) = self.offered_psk_session() else {
            return;
        };
        if self.max_version < TLS1_3 {
            return;
        }
        let hash = suite_hash(session.suite);
        let binders_length = 2 + 1 + hash.length();
        let truncated = hello.len() - binders_length;
        let binder = Self::psk_binder(&session.secret, hash, &hello[..truncated]);
        let binder_at = hello.len() - hash.length();
        hello[binder_at..].copy_from_slice(&binder);
    }

    /// A client whose pre-shared key the server chose resumes its session.
    pub(super) fn resume_with_psk(&mut self) -> Result<(), TlsError> {
        let Some(session) = self.offered_psk_session().cloned() else {
            return Err(failure(alert::ILLEGAL_PARAMETER, "bad psk"));
        };
        if suite_hash(session.suite) != suite_hash(self.suite) {
            return Err(failure(alert::ILLEGAL_PARAMETER, "bad psk"));
        }
        self.resumed = true;
        self.psk = Some(session.secret.clone());
        self.peer_certificates = session.peer_certificates.clone();
        self.session = Some(session);
        Ok(())
    }

    /// A server resumes the session a client's ticket holds when the
    /// client agreed to a key share alongside it and its suite's hash
    /// matches, and fails when the binder does not prove the key.
    pub(super) fn accept_psk(
        &mut self,
        ticket: &[u8],
        binder: &[u8],
        truncated_hello: &[u8],
    ) -> Result<(), TlsError> {
        if !self.psk_modes_offered {
            return Ok(());
        }
        let Some(session) = Self::open_ticket(ticket).filter(|held| {
            held.version == TLS1_3 && suite_hash(held.suite) == suite_hash(self.suite)
        }) else {
            return Ok(());
        };
        let expected = Self::psk_binder(&session.secret, suite_hash(self.suite), truncated_hello);
        if expected != binder {
            return Err(failure(alert::DECRYPT_ERROR, "binder does not verify"));
        }
        self.resumed = true;
        self.psk = Some(session.secret.clone());
        self.peer_certificates = session.peer_certificates.clone();
        self.session = Some(session);
        Ok(())
    }

    /// The session a TLS 1.3 connection runs in once established, which a
    /// ticket read later replaces.
    pub(super) fn hold_session(&mut self) {
        if self.session.is_some() {
            return;
        }
        self.session = Some(Session {
            version: TLS1_3,
            suite: self.suite,
            id: random_bytes::<32>().to_vec(),
            secret: Vec::new(),
            ticket: Vec::new(),
            age_add: 0,
            issued_at: now_in_milliseconds(),
            lifetime: TICKET_LIFETIME,
            peer_certificates: self.peer_certificates.clone(),
        });
    }

    /// A client keeps the session a TLS 1.3 NewSessionTicket gives.
    pub(super) fn read_new_session_ticket(&mut self, body: &[u8]) -> Result<(), TlsError> {
        let mut reader = Reader::new(body);
        let lifetime = reader.number(4)? as u32;
        let age_add = reader.number(4)? as u32;
        let nonce = reader.vector(1)?;
        let ticket = reader.vector(2)?.to_vec();
        reader.vector(2)?;
        let hash = suite_hash(self.suite);
        let session = Session {
            version: TLS1_3,
            suite: self.suite,
            id: random_bytes::<32>().to_vec(),
            secret: crypto::expand_label(
                hash,
                &self.resumption_secret,
                "resumption",
                nonce,
                hash.length(),
            ),
            ticket,
            age_add,
            issued_at: now_in_milliseconds(),
            lifetime,
            peer_certificates: self.peer_certificates.clone(),
        };
        self.session = Some(session.clone());
        self.new_sessions.push(session);
        Ok(())
    }

    /// A TLS 1.3 server sends tickets once the client's Finished is read.
    pub(super) fn send_session_tickets(&mut self, progress: &mut Progress) {
        let hash = suite_hash(self.suite);
        let resumption =
            crypto::derive_secret(hash, &self.master_secret, "res master", &self.transcript);
        let mut messages = Vec::new();
        // OpenSSL sends one ticket after a resumed handshake.
        let count = if self.resumed { 1 } else { TICKETS_SENT };
        for number in 0..count {
            let nonce = [number];
            let session = Session {
                version: TLS1_3,
                suite: self.suite,
                id: Vec::new(),
                secret: crypto::expand_label(
                    hash,
                    &resumption,
                    "resumption",
                    &nonce,
                    hash.length(),
                ),
                ticket: Vec::new(),
                age_add: 0,
                issued_at: now_in_milliseconds(),
                lifetime: TICKET_LIFETIME,
                peer_certificates: self.peer_certificates.clone(),
            };
            let mut body = TICKET_LIFETIME.to_be_bytes().to_vec();
            body.extend_from_slice(&random_bytes::<4>());
            body.extend(with_length(1, &nonce));
            body.extend(with_length(2, &Self::seal_ticket(&session)));
            body.extend(with_length(2, &[]));
            messages.extend(handshake_message(NEW_SESSION_TICKET, &body));
        }
        progress
            .outgoing
            .extend(self.sealed_records(HANDSHAKE, &messages));
    }
}
