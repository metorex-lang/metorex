// What a connection keeps to resume its session on a later connection: a
// TLS 1.2 master secret with the server's session id or ticket, or a
// TLS 1.3 pre-shared key with the ticket that names it.

use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug, PartialEq)]
pub struct Session {
    pub version: u16,
    pub suite: u16,
    pub id: Vec<u8>,
    /// The TLS 1.2 master secret, or the TLS 1.3 pre-shared key.
    pub secret: Vec<u8>,
    pub ticket: Vec<u8>,
    /// What a TLS 1.3 client adds to the ticket's age to hide it.
    pub age_add: u32,
    /// When the session began, in milliseconds since the epoch.
    pub issued_at: u64,
    /// The seconds the server keeps the session.
    pub lifetime: u32,
    /// The peer's chain, leaf first, which a resumed handshake does not
    /// send again.
    pub peer_certificates: Vec<Vec<u8>>,
}

pub fn now_in_milliseconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_millis() as u64)
        .unwrap_or(0)
}

fn push_vector(into: &mut Vec<u8>, width: usize, body: &[u8]) {
    let length = body.len().to_be_bytes();
    into.extend_from_slice(&length[length.len() - width..]);
    into.extend_from_slice(body);
}

struct Fields<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl<'a> Fields<'a> {
    fn take(&mut self, count: usize) -> Option<&'a [u8]> {
        let taken = self.bytes.get(self.at..self.at.checked_add(count)?)?;
        self.at += count;
        Some(taken)
    }

    fn number(&mut self, width: usize) -> Option<u64> {
        Some(
            self.take(width)?
                .iter()
                .fold(0u64, |held, byte| (held << 8) | u64::from(*byte)),
        )
    }

    fn vector(&mut self, width: usize) -> Option<&'a [u8]> {
        let length = self.number(width)? as usize;
        self.take(length)
    }
}

impl Session {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.version.to_be_bytes());
        bytes.extend_from_slice(&self.suite.to_be_bytes());
        push_vector(&mut bytes, 1, &self.id);
        push_vector(&mut bytes, 1, &self.secret);
        push_vector(&mut bytes, 2, &self.ticket);
        bytes.extend_from_slice(&self.age_add.to_be_bytes());
        bytes.extend_from_slice(&self.issued_at.to_be_bytes());
        bytes.extend_from_slice(&self.lifetime.to_be_bytes());
        let mut chain = Vec::new();
        for certificate in &self.peer_certificates {
            push_vector(&mut chain, 3, certificate);
        }
        push_vector(&mut bytes, 3, &chain);
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Session> {
        let mut fields = Fields { bytes, at: 0 };
        let version = fields.number(2)? as u16;
        let suite = fields.number(2)? as u16;
        let id = fields.vector(1)?.to_vec();
        let secret = fields.vector(1)?.to_vec();
        let ticket = fields.vector(2)?.to_vec();
        let age_add = fields.number(4)? as u32;
        let issued_at = fields.number(8)?;
        let lifetime = fields.number(4)? as u32;
        let mut chain = Fields {
            bytes: fields.vector(3)?,
            at: 0,
        };
        let mut peer_certificates = Vec::new();
        while chain.at < chain.bytes.len() {
            peer_certificates.push(chain.vector(3)?.to_vec());
        }
        if fields.at != bytes.len() {
            return None;
        }
        Some(Session {
            version,
            suite,
            id,
            secret,
            ticket,
            age_add,
            issued_at,
            lifetime,
            peer_certificates,
        })
    }
}
