// TLS 1.3 (RFC 8446) as a client or a server, with no I/O of its own: the
// bytes read from the peer are fed in, and the bytes to send and the
// application data read come back out. It offers and accepts one cipher
// suite, TLS_AES_128_GCM_SHA256, agrees keys with X25519, and proves the
// server's certificate with rsa_pss_rsae_sha256.

use super::crypto::{self, HASH_LENGTH, RsaKey};
use super::gcm::Gcm;
use super::x25519;
use crate::vm::native_methods::digest_algorithms::{hmac, sha256};

const TLS_AES_128_GCM_SHA256: u16 = 0x1301;
const GROUP_X25519: u16 = 0x001d;
const RSA_PSS_RSAE_SHA256: u16 = 0x0804;
const RSA_PKCS1_SHA256: u16 = 0x0401;

const CHANGE_CIPHER_SPEC: u8 = 20;
const ALERT: u8 = 21;
const HANDSHAKE: u8 = 22;
const APPLICATION_DATA: u8 = 23;

const CLIENT_HELLO: u8 = 1;
const SERVER_HELLO: u8 = 2;
const NEW_SESSION_TICKET: u8 = 4;
const ENCRYPTED_EXTENSIONS: u8 = 8;
const CERTIFICATE: u8 = 11;
const CERTIFICATE_REQUEST: u8 = 13;
const CERTIFICATE_VERIFY: u8 = 15;
const FINISHED: u8 = 20;
const KEY_UPDATE: u8 = 24;

const EXTENSION_SERVER_NAME: u16 = 0;
const EXTENSION_SUPPORTED_GROUPS: u16 = 10;
const EXTENSION_SIGNATURE_ALGORITHMS: u16 = 13;
const EXTENSION_SUPPORTED_VERSIONS: u16 = 43;
const EXTENSION_KEY_SHARE: u16 = 51;

/// The most plaintext one record carries.
const RECORD_LIMIT: usize = 16384;

/// Why a connection failed, with the alert sent or received for it.
#[derive(Clone, Debug)]
pub struct TlsError {
    pub message: String,
    /// The alert description TLS names the failure by.
    pub alert: u8,
}

pub mod alert {
    pub const CLOSE_NOTIFY: u8 = 0;
    pub const UNEXPECTED_MESSAGE: u8 = 10;
    pub const BAD_RECORD_MAC: u8 = 20;
    pub const HANDSHAKE_FAILURE: u8 = 40;
    pub const BAD_CERTIFICATE: u8 = 42;
    pub const CERTIFICATE_EXPIRED: u8 = 45;
    pub const CERTIFICATE_UNKNOWN: u8 = 46;
    pub const UNKNOWN_CA: u8 = 48;
    pub const DECODE_ERROR: u8 = 50;
    pub const DECRYPT_ERROR: u8 = 51;
    pub const PROTOCOL_VERSION: u8 = 70;
    pub const INTERNAL_ERROR: u8 = 80;
    pub const MISSING_EXTENSION: u8 = 109;
}

fn failure(alert: u8, message: &str) -> TlsError {
    TlsError {
        message: message.to_string(),
        alert,
    }
}

/// What the handshake waits for next.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Stage {
    ClientStart,
    WaitServerHello,
    WaitEncryptedExtensions,
    WaitCertificate,
    WaitCertificateVerify,
    WaitServerFinished,
    /// The server's chain has been read and proved, and the program decides
    /// whether it trusts it.
    WaitCertificateDecision,
    ServerStart,
    WaitClientHello,
    WaitClientFinished,
    Established,
    Closed,
}

/// The keys one direction of the connection protects records with.
struct Direction {
    gcm: Gcm,
    iv: [u8; 12],
    sequence: u64,
}

impl Direction {
    fn new(secret: &[u8]) -> Direction {
        let key = crypto::expand_label(secret, "key", &[], 16);
        let iv_bytes = crypto::expand_label(secret, "iv", &[], 12);
        let mut iv = [0u8; 12];
        iv.copy_from_slice(&iv_bytes);
        Direction {
            gcm: Gcm::new(&key).expect("a 16-byte key"),
            iv,
            sequence: 0,
        }
    }

    fn nonce(&mut self) -> [u8; 12] {
        let mut nonce = self.iv;
        for (at, byte) in self.sequence.to_be_bytes().iter().enumerate() {
            nonce[4 + at] ^= byte;
        }
        self.sequence += 1;
        nonce
    }
}

/// What a feed of bytes moved the connection to.
#[derive(Debug, Default)]
pub struct Progress {
    /// Bytes to send to the peer.
    pub outgoing: Vec<u8>,
    /// Application data the peer sent.
    pub plaintext: Vec<u8>,
    /// Whether the peer closed its side.
    pub peer_closed: bool,
}

pub struct Connection {
    is_client: bool,
    stage: Stage,
    server_name: Option<String>,
    certificates: Vec<Vec<u8>>,
    private_key: Option<RsaKey>,
    secret_key: [u8; 32],
    transcript: Vec<u8>,
    handshake_secret: Vec<u8>,
    client_handshake_secret: Vec<u8>,
    server_handshake_secret: Vec<u8>,
    master_secret: Vec<u8>,
    reading: Option<Direction>,
    writing: Option<Direction>,
    /// Bytes read that do not yet make a whole record.
    unread: Vec<u8>,
    /// Handshake bytes that do not yet make a whole message.
    handshake_buffer: Vec<u8>,
    /// The chain the peer sent, leaf first.
    pub peer_certificates: Vec<Vec<u8>>,
    /// The server name the client asked for, as the server read it.
    pub requested_name: Option<String>,
    /// The client's key share, held while a server waits to answer.
    client_share: Option<[u8; 32]>,
    /// The client application secret, kept until the client's Finished is
    /// checked.
    pending_client_secret: Option<Vec<u8>>,
}

fn random_bytes<const N: usize>() -> [u8; N] {
    let mut bytes = [0u8; N];
    if let Ok(mut source) = std::fs::File::open("/dev/urandom") {
        use std::io::Read;
        let _ = source.read_exact(&mut bytes);
    }
    bytes
}

fn push_u16(into: &mut Vec<u8>, value: u16) {
    into.extend_from_slice(&value.to_be_bytes());
}

fn push_u24(into: &mut Vec<u8>, value: usize) {
    into.extend_from_slice(&(value as u32).to_be_bytes()[1..]);
}

/// The bytes with a length in front, written in `width` bytes.
fn with_length(width: usize, body: &[u8]) -> Vec<u8> {
    let mut written = Vec::with_capacity(width + body.len());
    let length = body.len().to_be_bytes();
    written.extend_from_slice(&length[length.len() - width..]);
    written.extend_from_slice(body);
    written
}

fn extension(kind: u16, body: &[u8]) -> Vec<u8> {
    let mut written = Vec::new();
    push_u16(&mut written, kind);
    written.extend(with_length(2, body));
    written
}

fn handshake_message(kind: u8, body: &[u8]) -> Vec<u8> {
    let mut written = vec![kind];
    push_u24(&mut written, body.len());
    written.extend_from_slice(body);
    written
}

/// A reader over a message's bytes that fails rather than reading past
/// their end.
struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8]) -> Reader<'a> {
        Reader { bytes, at: 0 }
    }

    fn take(&mut self, count: usize) -> Result<&'a [u8], TlsError> {
        if self.at + count > self.bytes.len() {
            return Err(failure(alert::DECODE_ERROR, "decode error"));
        }
        let taken = &self.bytes[self.at..self.at + count];
        self.at += count;
        Ok(taken)
    }

    fn number(&mut self, width: usize) -> Result<usize, TlsError> {
        Ok(self
            .take(width)?
            .iter()
            .fold(0usize, |held, byte| (held << 8) | *byte as usize))
    }

    fn vector(&mut self, width: usize) -> Result<&'a [u8], TlsError> {
        let length = self.number(width)?;
        self.take(length)
    }

    fn done(&self) -> bool {
        self.at >= self.bytes.len()
    }
}

/// Each extension of a list, as its type and body.
fn extensions(bytes: &[u8]) -> Result<Vec<(u16, &[u8])>, TlsError> {
    let mut reader = Reader::new(bytes);
    let mut found = Vec::new();
    while !reader.done() {
        let kind = reader.number(2)? as u16;
        let body = reader.vector(2)?;
        found.push((kind, body));
    }
    Ok(found)
}

impl Connection {
    pub fn client(server_name: Option<String>) -> Connection {
        Connection::new(true, server_name, Vec::new(), None)
    }

    pub fn server(certificates: Vec<Vec<u8>>, private_key: RsaKey) -> Connection {
        Connection::new(false, None, certificates, Some(private_key))
    }

    fn new(
        is_client: bool,
        server_name: Option<String>,
        certificates: Vec<Vec<u8>>,
        private_key: Option<RsaKey>,
    ) -> Connection {
        Connection {
            is_client,
            stage: if is_client {
                Stage::ClientStart
            } else {
                Stage::ServerStart
            },
            server_name,
            certificates,
            private_key,
            secret_key: random_bytes(),
            transcript: Vec::new(),
            handshake_secret: Vec::new(),
            client_handshake_secret: Vec::new(),
            server_handshake_secret: Vec::new(),
            master_secret: Vec::new(),
            reading: None,
            writing: None,
            unread: Vec::new(),
            handshake_buffer: Vec::new(),
            peer_certificates: Vec::new(),
            requested_name: None,
            client_share: None,
            pending_client_secret: None,
        }
    }

    pub fn is_established(&self) -> bool {
        self.stage == Stage::Established
    }

    pub fn waits_for_certificate_decision(&self) -> bool {
        self.stage == Stage::WaitCertificateDecision
    }

    pub fn is_closed(&self) -> bool {
        self.stage == Stage::Closed
    }

    /// The records a client opens the handshake with.
    pub fn start(&mut self) -> Vec<u8> {
        if self.stage != Stage::ClientStart {
            self.stage = if self.is_client {
                self.stage
            } else {
                Stage::WaitClientHello
            };
            return Vec::new();
        }
        let hello = self.client_hello();
        self.transcript.extend_from_slice(&hello);
        self.stage = Stage::WaitServerHello;
        self.plain_record(HANDSHAKE, &hello)
    }

    fn client_hello(&self) -> Vec<u8> {
        let mut body = Vec::new();
        push_u16(&mut body, 0x0303);
        body.extend_from_slice(&random_bytes::<32>());
        body.extend(with_length(1, &random_bytes::<32>()));
        body.extend(with_length(2, &TLS_AES_128_GCM_SHA256.to_be_bytes()));
        body.extend(with_length(1, &[0]));
        let mut listed = Vec::new();
        if let Some(name) = &self.server_name
            && name.parse::<std::net::IpAddr>().is_err()
        {
            let mut entry = vec![0u8];
            entry.extend(with_length(2, name.as_bytes()));
            listed.extend(extension(EXTENSION_SERVER_NAME, &with_length(2, &entry)));
        }
        listed.extend(extension(
            EXTENSION_SUPPORTED_VERSIONS,
            &with_length(1, &0x0304u16.to_be_bytes()),
        ));
        listed.extend(extension(
            EXTENSION_SUPPORTED_GROUPS,
            &with_length(2, &GROUP_X25519.to_be_bytes()),
        ));
        let mut algorithms = Vec::new();
        push_u16(&mut algorithms, RSA_PSS_RSAE_SHA256);
        push_u16(&mut algorithms, RSA_PKCS1_SHA256);
        listed.extend(extension(
            EXTENSION_SIGNATURE_ALGORITHMS,
            &with_length(2, &algorithms),
        ));
        let mut share = Vec::new();
        push_u16(&mut share, GROUP_X25519);
        share.extend(with_length(2, &x25519::public_key(&self.secret_key)));
        listed.extend(extension(EXTENSION_KEY_SHARE, &with_length(2, &share)));
        body.extend(with_length(2, &listed));
        handshake_message(CLIENT_HELLO, &body)
    }

    fn plain_record(&self, kind: u8, body: &[u8]) -> Vec<u8> {
        let mut written = Vec::new();
        for chunk in body.chunks(RECORD_LIMIT) {
            written.push(kind);
            push_u16(&mut written, 0x0303);
            written.extend(with_length(2, chunk));
        }
        written
    }

    fn sealed_records(&mut self, kind: u8, body: &[u8]) -> Vec<u8> {
        let mut written = Vec::new();
        let chunks: Vec<&[u8]> = if body.is_empty() {
            vec![&[][..]]
        } else {
            body.chunks(RECORD_LIMIT).collect()
        };
        for chunk in chunks {
            let Some(direction) = self.writing.as_mut() else {
                return self.plain_record(kind, body);
            };
            let mut inner = chunk.to_vec();
            inner.push(kind);
            let mut header = vec![APPLICATION_DATA];
            push_u16(&mut header, 0x0303);
            push_u16(&mut header, (inner.len() + 16) as u16);
            let nonce = direction.nonce();
            let sealed = direction.gcm.seal(&nonce, &header, &inner);
            written.extend_from_slice(&header);
            written.extend_from_slice(&sealed);
        }
        written
    }

    /// The records carrying application data.
    pub fn write(&mut self, plaintext: &[u8]) -> Result<Vec<u8>, TlsError> {
        if self.stage != Stage::Established {
            return Err(failure(
                alert::UNEXPECTED_MESSAGE,
                "the handshake has not finished",
            ));
        }
        Ok(self.sealed_records(APPLICATION_DATA, plaintext))
    }

    /// The alert that closes the connection's sending side.
    pub fn close(&mut self) -> Vec<u8> {
        let record = self.alert_record(alert::CLOSE_NOTIFY);
        self.stage = Stage::Closed;
        record
    }

    /// The alert record telling the peer of a failure.
    pub fn alert_record(&mut self, description: u8) -> Vec<u8> {
        let level = if description == alert::CLOSE_NOTIFY {
            1
        } else {
            2
        };
        self.sealed_records(ALERT, &[level, description])
    }

    /// Answer the chain check a client paused for: the handshake carries on
    /// when it is trusted and ends with the alert given when it is not.
    pub fn decide_certificate(&mut self, refused: Option<u8>) -> Result<Progress, TlsError> {
        if self.stage != Stage::WaitCertificateDecision {
            return Ok(Progress::default());
        }
        if let Some(description) = refused {
            let record = self.alert_record(description);
            self.stage = Stage::Closed;
            return Ok(Progress {
                outgoing: record,
                ..Progress::default()
            });
        }
        self.stage = Stage::WaitServerFinished;
        // The Finished that came with the chain, and any record after it,
        // may already have been read.
        let mut progress = Progress::default();
        self.read_handshake_messages(&mut progress)?;
        let later = self.feed(&[])?;
        progress.outgoing.extend(later.outgoing);
        progress.plaintext.extend(later.plaintext);
        progress.peer_closed |= later.peer_closed;
        Ok(progress)
    }

    /// Feed bytes read from the peer.
    pub fn feed(&mut self, bytes: &[u8]) -> Result<Progress, TlsError> {
        self.unread.extend_from_slice(bytes);
        let mut progress = Progress::default();
        if self.stage == Stage::ServerStart {
            self.stage = Stage::WaitClientHello;
        }
        loop {
            if self.stage == Stage::WaitCertificateDecision || self.stage == Stage::Closed {
                break;
            }
            if self.unread.len() < 5 {
                break;
            }
            let length = u16::from_be_bytes([self.unread[3], self.unread[4]]) as usize;
            if length > RECORD_LIMIT + 256 {
                return Err(failure(alert::DECODE_ERROR, "record overflow"));
            }
            if self.unread.len() < 5 + length {
                break;
            }
            let record: Vec<u8> = self.unread.drain(..5 + length).collect();
            let kind = record[0];
            let (kind, body) = match (kind, self.reading.as_mut()) {
                (CHANGE_CIPHER_SPEC, _) => continue,
                (APPLICATION_DATA, Some(direction)) => {
                    let nonce = direction.nonce();
                    let Some(mut inner) = direction.gcm.open(&nonce, &record[..5], &record[5..])
                    else {
                        return Err(failure(alert::BAD_RECORD_MAC, "bad record mac"));
                    };
                    while inner.last() == Some(&0) {
                        inner.pop();
                    }
                    let Some(kind) = inner.pop() else {
                        return Err(failure(alert::UNEXPECTED_MESSAGE, "unexpected message"));
                    };
                    (kind, inner)
                }
                (_, _) => (kind, record[5..].to_vec()),
            };
            match kind {
                HANDSHAKE => {
                    self.handshake_buffer.extend_from_slice(&body);
                    self.read_handshake_messages(&mut progress)?;
                }
                APPLICATION_DATA if self.stage == Stage::Established => {
                    progress.plaintext.extend_from_slice(&body);
                }
                ALERT => {
                    let description = body.get(1).copied().unwrap_or(0);
                    if description == alert::CLOSE_NOTIFY {
                        progress.peer_closed = true;
                        self.stage = Stage::Closed;
                        break;
                    }
                    self.stage = Stage::Closed;
                    return Err(TlsError {
                        message: format!(
                            "tlsv1 alert {} (SSL alert number {description})",
                            alert_name(description)
                        ),
                        alert: description,
                    });
                }
                _ => return Err(failure(alert::UNEXPECTED_MESSAGE, "unexpected message")),
            }
        }
        Ok(progress)
    }

    fn read_handshake_messages(&mut self, progress: &mut Progress) -> Result<(), TlsError> {
        loop {
            if self.stage == Stage::WaitCertificateDecision || self.stage == Stage::Closed {
                return Ok(());
            }
            if self.handshake_buffer.len() < 4 {
                return Ok(());
            }
            let length = (self.handshake_buffer[1] as usize) << 16
                | (self.handshake_buffer[2] as usize) << 8
                | self.handshake_buffer[3] as usize;
            if self.handshake_buffer.len() < 4 + length {
                return Ok(());
            }
            let message: Vec<u8> = self.handshake_buffer.drain(..4 + length).collect();
            self.handle_handshake(&message, progress)?;
        }
    }

    fn handle_handshake(
        &mut self,
        message: &[u8],
        progress: &mut Progress,
    ) -> Result<(), TlsError> {
        let kind = message[0];
        let body = &message[4..];
        match (self.stage, kind) {
            (Stage::WaitServerHello, SERVER_HELLO) => self.read_server_hello(message, body),
            (Stage::WaitEncryptedExtensions, ENCRYPTED_EXTENSIONS) => {
                self.transcript.extend_from_slice(message);
                self.stage = Stage::WaitCertificate;
                Ok(())
            }
            (Stage::WaitCertificate, CERTIFICATE_REQUEST) => Err(failure(
                alert::HANDSHAKE_FAILURE,
                "client certificates are not supported",
            )),
            (Stage::WaitCertificate, CERTIFICATE) => self.read_certificate(message, body),
            (Stage::WaitCertificateVerify, CERTIFICATE_VERIFY) => {
                self.read_certificate_verify(message, body)
            }
            (Stage::WaitServerFinished, FINISHED) => {
                self.read_server_finished(message, body, progress)
            }
            (Stage::WaitClientHello, CLIENT_HELLO) => {
                self.read_client_hello(message, body, progress)
            }
            (Stage::WaitClientFinished, FINISHED) => self.read_client_finished(message, body),
            (Stage::Established, NEW_SESSION_TICKET) => Ok(()),
            (Stage::Established, KEY_UPDATE) => Err(failure(
                alert::UNEXPECTED_MESSAGE,
                "key updates are not supported",
            )),
            _ => Err(failure(alert::UNEXPECTED_MESSAGE, "unexpected message")),
        }
    }

    fn read_server_hello(&mut self, message: &[u8], body: &[u8]) -> Result<(), TlsError> {
        let mut reader = Reader::new(body);
        reader.take(2)?;
        reader.take(32)?;
        reader.vector(1)?;
        let suite = reader.number(2)? as u16;
        reader.take(1)?;
        if suite != TLS_AES_128_GCM_SHA256 {
            return Err(failure(alert::HANDSHAKE_FAILURE, "no shared cipher"));
        }
        let mut peer_share = None;
        let mut version = 0x0303;
        for (kind, extension_body) in extensions(reader.vector(2)?)? {
            match kind {
                EXTENSION_SUPPORTED_VERSIONS => version = Reader::new(extension_body).number(2)?,
                EXTENSION_KEY_SHARE => {
                    let mut share = Reader::new(extension_body);
                    let group = share.number(2)? as u16;
                    let key = share.vector(2)?;
                    if group == GROUP_X25519 && key.len() == 32 {
                        let mut held = [0u8; 32];
                        held.copy_from_slice(key);
                        peer_share = Some(held);
                    }
                }
                _ => {}
            }
        }
        if version != 0x0304 {
            return Err(failure(alert::PROTOCOL_VERSION, "unsupported protocol"));
        }
        let Some(peer_share) = peer_share else {
            return Err(failure(alert::HANDSHAKE_FAILURE, "no shared group"));
        };
        self.transcript.extend_from_slice(message);
        self.enter_handshake_keys(&peer_share);
        self.reading = Some(Direction::new(&self.server_handshake_secret.clone()));
        self.writing = Some(Direction::new(&self.client_handshake_secret.clone()));
        self.stage = Stage::WaitEncryptedExtensions;
        Ok(())
    }

    fn enter_handshake_keys(&mut self, peer_share: &[u8; 32]) {
        let shared = x25519::x25519(&self.secret_key, peer_share);
        let early = crypto::hkdf_extract(&[], &[0u8; HASH_LENGTH]);
        let derived = crypto::derive_secret(&early, "derived", &[]);
        self.handshake_secret = crypto::hkdf_extract(&derived, &shared);
        self.client_handshake_secret =
            crypto::derive_secret(&self.handshake_secret, "c hs traffic", &self.transcript);
        self.server_handshake_secret =
            crypto::derive_secret(&self.handshake_secret, "s hs traffic", &self.transcript);
        let derived = crypto::derive_secret(&self.handshake_secret, "derived", &[]);
        self.master_secret = crypto::hkdf_extract(&derived, &[0u8; HASH_LENGTH]);
    }

    fn read_certificate(&mut self, message: &[u8], body: &[u8]) -> Result<(), TlsError> {
        let mut reader = Reader::new(body);
        reader.vector(1)?;
        let mut list = Reader::new(reader.vector(3)?);
        let mut chain = Vec::new();
        while !list.done() {
            chain.push(list.vector(3)?.to_vec());
            list.vector(2)?;
        }
        if chain.is_empty() {
            return Err(failure(
                alert::BAD_CERTIFICATE,
                "peer did not return a certificate",
            ));
        }
        self.peer_certificates = chain;
        self.transcript.extend_from_slice(message);
        self.stage = Stage::WaitCertificateVerify;
        Ok(())
    }

    fn certificate_verify_content(&self, server: bool) -> Vec<u8> {
        let mut content = vec![0x20u8; 64];
        content.extend_from_slice(if server {
            b"TLS 1.3, server CertificateVerify"
        } else {
            b"TLS 1.3, client CertificateVerify"
        });
        content.push(0);
        content.extend_from_slice(&sha256(&self.transcript));
        content
    }

    fn read_certificate_verify(&mut self, message: &[u8], body: &[u8]) -> Result<(), TlsError> {
        let mut reader = Reader::new(body);
        let algorithm = reader.number(2)? as u16;
        let signature = reader.vector(2)?;
        let leaf = &self.peer_certificates[0];
        let Some(key) = super::certificate::rsa_public_key(leaf) else {
            return Err(failure(
                alert::BAD_CERTIFICATE,
                "unsupported certificate key",
            ));
        };
        let content = self.certificate_verify_content(true);
        let proved = match algorithm {
            RSA_PSS_RSAE_SHA256 => key.verify_pss(&content, signature),
            _ => false,
        };
        if !proved {
            return Err(failure(alert::DECRYPT_ERROR, "bad signature"));
        }
        self.transcript.extend_from_slice(message);
        self.stage = Stage::WaitCertificateDecision;
        Ok(())
    }

    fn finished_data(&self, secret: &[u8]) -> Vec<u8> {
        let key = crypto::expand_label(secret, "finished", &[], HASH_LENGTH);
        hmac("SHA256", &key, &sha256(&self.transcript)).unwrap_or_default()
    }

    fn read_server_finished(
        &mut self,
        message: &[u8],
        body: &[u8],
        progress: &mut Progress,
    ) -> Result<(), TlsError> {
        if body != self.finished_data(&self.server_handshake_secret) {
            return Err(failure(alert::DECRYPT_ERROR, "digest check failed"));
        }
        self.transcript.extend_from_slice(message);
        let client_application =
            crypto::derive_secret(&self.master_secret, "c ap traffic", &self.transcript);
        let server_application =
            crypto::derive_secret(&self.master_secret, "s ap traffic", &self.transcript);
        let finished =
            handshake_message(FINISHED, &self.finished_data(&self.client_handshake_secret));
        self.transcript.extend_from_slice(&finished);
        let records = self.sealed_records(HANDSHAKE, &finished);
        progress.outgoing.extend(records);
        self.reading = Some(Direction::new(&server_application));
        self.writing = Some(Direction::new(&client_application));
        self.stage = Stage::Established;
        Ok(())
    }

    fn read_client_hello(
        &mut self,
        message: &[u8],
        body: &[u8],
        progress: &mut Progress,
    ) -> Result<(), TlsError> {
        let mut reader = Reader::new(body);
        reader.take(2)?;
        reader.take(32)?;
        let session = reader.vector(1)?.to_vec();
        let suites = reader.vector(2)?;
        reader.vector(1)?;
        let offered = suites
            .chunks(2)
            .any(|suite| suite == TLS_AES_128_GCM_SHA256.to_be_bytes());
        let mut speaks_1_3 = false;
        let mut share = None;
        let mut signs_with_pss = false;
        if !reader.done() {
            for (kind, extension_body) in extensions(reader.vector(2)?)? {
                match kind {
                    EXTENSION_SUPPORTED_VERSIONS => {
                        let versions = Reader::new(extension_body).vector(1)?;
                        speaks_1_3 = versions.chunks(2).any(|version| version == [3, 4]);
                    }
                    EXTENSION_SIGNATURE_ALGORITHMS => {
                        let algorithms = Reader::new(extension_body).vector(2)?;
                        signs_with_pss = algorithms
                            .chunks(2)
                            .any(|algorithm| algorithm == RSA_PSS_RSAE_SHA256.to_be_bytes());
                    }
                    EXTENSION_KEY_SHARE => {
                        let mut shares = Reader::new(Reader::new(extension_body).vector(2)?);
                        while !shares.done() {
                            let group = shares.number(2)? as u16;
                            let key = shares.vector(2)?;
                            if group == GROUP_X25519 && key.len() == 32 {
                                let mut held = [0u8; 32];
                                held.copy_from_slice(key);
                                share = Some(held);
                            }
                        }
                    }
                    EXTENSION_SERVER_NAME => {
                        let mut names = Reader::new(Reader::new(extension_body).vector(2)?);
                        if names.number(1)? == 0 {
                            let name = names.vector(2)?;
                            self.requested_name = Some(String::from_utf8_lossy(name).into_owned());
                        }
                    }
                    _ => {}
                }
            }
        }
        if !speaks_1_3 {
            return Err(failure(alert::PROTOCOL_VERSION, "unsupported protocol"));
        }
        if !offered {
            return Err(failure(alert::HANDSHAKE_FAILURE, "no shared cipher"));
        }
        let Some(peer_share) = share else {
            return Err(failure(alert::MISSING_EXTENSION, "no suitable key share"));
        };
        if !signs_with_pss {
            return Err(failure(
                alert::HANDSHAKE_FAILURE,
                "no suitable signature algorithm",
            ));
        }
        self.transcript.extend_from_slice(message);
        self.client_share = Some(peer_share);
        let hello = self.server_hello(&session);
        self.transcript.extend_from_slice(&hello);
        progress
            .outgoing
            .extend(self.plain_record(HANDSHAKE, &hello));
        self.enter_handshake_keys(&peer_share);
        self.reading = Some(Direction::new(&self.client_handshake_secret.clone()));
        self.writing = Some(Direction::new(&self.server_handshake_secret.clone()));
        let mut flight = handshake_message(ENCRYPTED_EXTENSIONS, &with_length(2, &[]));
        self.transcript.extend_from_slice(&flight);
        let certificate = self.certificate_message();
        self.transcript.extend_from_slice(&certificate);
        flight.extend_from_slice(&certificate);
        let verify = self.certificate_verify_message()?;
        self.transcript.extend_from_slice(&verify);
        flight.extend_from_slice(&verify);
        let finished =
            handshake_message(FINISHED, &self.finished_data(&self.server_handshake_secret));
        self.transcript.extend_from_slice(&finished);
        flight.extend_from_slice(&finished);
        progress
            .outgoing
            .extend(self.sealed_records(HANDSHAKE, &flight));
        let server_application =
            crypto::derive_secret(&self.master_secret, "s ap traffic", &self.transcript);
        self.pending_client_secret = Some(crypto::derive_secret(
            &self.master_secret,
            "c ap traffic",
            &self.transcript,
        ));
        self.writing = Some(Direction::new(&server_application));
        self.stage = Stage::WaitClientFinished;
        Ok(())
    }

    fn server_hello(&self, session: &[u8]) -> Vec<u8> {
        let mut body = Vec::new();
        push_u16(&mut body, 0x0303);
        body.extend_from_slice(&random_bytes::<32>());
        body.extend(with_length(1, session));
        push_u16(&mut body, TLS_AES_128_GCM_SHA256);
        body.push(0);
        let mut listed = extension(EXTENSION_SUPPORTED_VERSIONS, &0x0304u16.to_be_bytes());
        let mut share = Vec::new();
        push_u16(&mut share, GROUP_X25519);
        share.extend(with_length(2, &x25519::public_key(&self.secret_key)));
        listed.extend(extension(EXTENSION_KEY_SHARE, &share));
        body.extend(with_length(2, &listed));
        handshake_message(SERVER_HELLO, &body)
    }

    fn certificate_message(&self) -> Vec<u8> {
        let mut list = Vec::new();
        for certificate in &self.certificates {
            list.extend(with_length(3, certificate));
            list.extend(with_length(2, &[]));
        }
        let mut body = with_length(1, &[]);
        body.extend(with_length(3, &list));
        handshake_message(CERTIFICATE, &body)
    }

    fn certificate_verify_message(&self) -> Result<Vec<u8>, TlsError> {
        let Some(key) = &self.private_key else {
            return Err(failure(alert::INTERNAL_ERROR, "no private key"));
        };
        let content = self.certificate_verify_content(true);
        let Some(signature) = key.sign_pss(&content, &random_bytes::<HASH_LENGTH>()) else {
            return Err(failure(alert::INTERNAL_ERROR, "the key could not sign"));
        };
        let mut body = Vec::new();
        push_u16(&mut body, RSA_PSS_RSAE_SHA256);
        body.extend(with_length(2, &signature));
        Ok(handshake_message(CERTIFICATE_VERIFY, &body))
    }

    fn read_client_finished(&mut self, message: &[u8], body: &[u8]) -> Result<(), TlsError> {
        if body != self.finished_data(&self.client_handshake_secret) {
            return Err(failure(alert::DECRYPT_ERROR, "digest check failed"));
        }
        self.transcript.extend_from_slice(message);
        let Some(client_application) = self.pending_client_secret.take() else {
            return Err(failure(alert::INTERNAL_ERROR, "no client secret"));
        };
        self.reading = Some(Direction::new(&client_application));
        self.stage = Stage::Established;
        Ok(())
    }
}

/// The name OpenSSL gives an alert description in its messages.
pub fn alert_name(description: u8) -> &'static str {
    match description {
        alert::CLOSE_NOTIFY => "close notify",
        alert::UNEXPECTED_MESSAGE => "unexpected message",
        alert::BAD_RECORD_MAC => "bad record mac",
        alert::HANDSHAKE_FAILURE => "handshake failure",
        alert::BAD_CERTIFICATE => "bad certificate",
        alert::CERTIFICATE_EXPIRED => "certificate expired",
        alert::CERTIFICATE_UNKNOWN => "certificate unknown",
        alert::UNKNOWN_CA => "unknown ca",
        alert::DECODE_ERROR => "decode error",
        alert::DECRYPT_ERROR => "decrypt error",
        alert::PROTOCOL_VERSION => "protocol version",
        alert::INTERNAL_ERROR => "internal error",
        alert::MISSING_EXTENSION => "missing extension",
        _ => "unknown",
    }
}
