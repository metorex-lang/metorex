// TLS 1.3 (RFC 8446) and TLS 1.2 (RFC 5246) as a client or a server, with
// no I/O of its own: the bytes read from the peer are fed in, and the bytes
// to send and the application data read come back out. TLS 1.3 speaks one
// cipher suites, TLS_AES_128_GCM_SHA256 and TLS_CHACHA20_POLY1305_SHA256,
// preferring the first, and TLS 1.2 speaks
// TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256. Both agree keys with X25519 and
// prove the server's certificate with rsa_pss_rsae_sha256, and TLS 1.2 also
// with rsa_pkcs1_sha256. A server can ask the client for a certificate,
// which the client proves the same way.

use super::chacha20_poly1305::ChaCha20Poly1305;
use super::crypto::{self, HASH_LENGTH, Hash, RsaKey};
use super::gcm::Gcm;
use super::x25519;
use crate::vm::native_methods::digest_algorithms::sha256;

const TLS_AES_128_GCM_SHA256: u16 = 0x1301;
const TLS_AES_256_GCM_SHA384: u16 = 0x1302;
const TLS_CHACHA20_POLY1305_SHA256: u16 = 0x1303;
const TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256: u16 = 0xC02F;
/// The cipher suite value a client lists to ask for secure renegotiation
/// without sending the extension (RFC 5746).
const TLS_EMPTY_RENEGOTIATION_INFO_SCSV: u16 = 0x00FF;
pub const TLS1_2: u16 = 0x0303;
pub const TLS1_3: u16 = 0x0304;
/// The last 8 bytes of the random a server able to speak TLS 1.3 sends when
/// it agrees to TLS 1.2 (RFC 8446, section 4.1.3).
const DOWNGRADE_TO_TLS1_2: &[u8; 8] = b"DOWNGRD\x01";
/// The curve_type of a ServerKeyExchange naming its group.
const NAMED_CURVE: u8 = 3;
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
const SERVER_KEY_EXCHANGE: u8 = 12;
const CERTIFICATE_REQUEST: u8 = 13;
const SERVER_HELLO_DONE: u8 = 14;
const CERTIFICATE_VERIFY: u8 = 15;
const CLIENT_KEY_EXCHANGE: u8 = 16;
const FINISHED: u8 = 20;
const KEY_UPDATE: u8 = 24;

const EXTENSION_SERVER_NAME: u16 = 0;
const EXTENSION_SUPPORTED_GROUPS: u16 = 10;
const EXTENSION_EC_POINT_FORMATS: u16 = 11;
const EXTENSION_SIGNATURE_ALGORITHMS: u16 = 13;
const EXTENSION_SUPPORTED_VERSIONS: u16 = 43;
const EXTENSION_KEY_SHARE: u16 = 51;
const EXTENSION_RENEGOTIATION_INFO: u16 = 0xff01;

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
    pub const ILLEGAL_PARAMETER: u8 = 47;
    pub const BAD_CERTIFICATE: u8 = 42;
    pub const CERTIFICATE_EXPIRED: u8 = 45;
    pub const CERTIFICATE_UNKNOWN: u8 = 46;
    pub const UNKNOWN_CA: u8 = 48;
    pub const DECODE_ERROR: u8 = 50;
    pub const DECRYPT_ERROR: u8 = 51;
    pub const PROTOCOL_VERSION: u8 = 70;
    pub const INTERNAL_ERROR: u8 = 80;
    pub const MISSING_EXTENSION: u8 = 109;
    pub const CERTIFICATE_REQUIRED: u8 = 116;
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
    WaitServerKeyExchange,
    WaitServerHelloDone,
    WaitServerFinished,
    /// The server's chain has been read and proved, and the program decides
    /// whether it trusts it.
    WaitCertificateDecision,
    ServerStart,
    WaitClientHello,
    WaitClientCertificate,
    WaitClientKeyExchange,
    WaitClientCertificateVerify,
    WaitClientFinished,
    Established,
    Closed,
}

/// The cipher sealing records, by the suite agreed on.
enum Aead {
    Gcm(Gcm),
    ChaCha20Poly1305(ChaCha20Poly1305),
}

impl Aead {
    fn seal(&self, nonce: &[u8; 12], additional: &[u8], plaintext: &[u8]) -> Vec<u8> {
        match self {
            Aead::Gcm(cipher) => cipher.seal(nonce, additional, plaintext),
            Aead::ChaCha20Poly1305(cipher) => cipher.seal(nonce, additional, plaintext),
        }
    }

    fn open(&self, nonce: &[u8; 12], additional: &[u8], sealed: &[u8]) -> Option<Vec<u8>> {
        match self {
            Aead::Gcm(cipher) => cipher.open(nonce, additional, sealed),
            Aead::ChaCha20Poly1305(cipher) => cipher.open(nonce, additional, sealed),
        }
    }
}

/// The hash a TLS 1.3 suite derives its keys and checks its transcript
/// with.
fn suite_hash(suite: u16) -> Hash {
    if suite == TLS_AES_256_GCM_SHA384 {
        Hash::Sha384
    } else {
        Hash::Sha256
    }
}

/// The keys one direction of the connection protects records with.
struct Direction {
    gcm: Aead,
    iv: [u8; 12],
    sequence: u64,
    /// TLS 1.2 records keep their content type in the header, send the
    /// nonce's last 8 bytes in front of the ciphertext, and authenticate
    /// the sequence number and the plaintext's length (RFC 5288).
    explicit_nonce: bool,
}

/// The tag GCM adds to each sealed record.
const TAG_LENGTH: usize = 16;
/// The nonce bytes a TLS 1.2 record carries in front of its ciphertext.
const EXPLICIT_NONCE_LENGTH: usize = 8;

impl Direction {
    fn new(secret: &[u8], suite: u16) -> Direction {
        let hash = suite_hash(suite);
        let iv_bytes = crypto::expand_label(hash, secret, "iv", &[], 12);
        let mut iv = [0u8; 12];
        iv.copy_from_slice(&iv_bytes);
        let gcm = match suite {
            TLS_CHACHA20_POLY1305_SHA256 => {
                let key = crypto::expand_label(hash, secret, "key", &[], 32);
                Aead::ChaCha20Poly1305(ChaCha20Poly1305::new(&key).expect("a 32-byte key"))
            }
            TLS_AES_256_GCM_SHA384 => {
                let key = crypto::expand_label(hash, secret, "key", &[], 32);
                Aead::Gcm(Gcm::new(&key).expect("a 32-byte key"))
            }
            _ => {
                let key = crypto::expand_label(hash, secret, "key", &[], 16);
                Aead::Gcm(Gcm::new(&key).expect("a 16-byte key"))
            }
        };
        Direction {
            gcm,
            iv,
            sequence: 0,
            explicit_nonce: false,
        }
    }

    /// A TLS 1.2 direction, from its key and the 4-byte salt the key block
    /// gives as its IV.
    fn tls12(key: &[u8], salt: &[u8]) -> Direction {
        let mut iv = [0u8; 12];
        iv[..4].copy_from_slice(salt);
        Direction {
            gcm: Aead::Gcm(Gcm::new(key).expect("a 16-byte key")),
            iv,
            sequence: 0,
            explicit_nonce: true,
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

    /// The additional data a TLS 1.2 record authenticates.
    fn tls12_additional(sequence: u64, kind: u8, length: usize) -> Vec<u8> {
        let mut additional = sequence.to_be_bytes().to_vec();
        additional.push(kind);
        push_u16(&mut additional, TLS1_2);
        push_u16(&mut additional, length as u16);
        additional
    }

    /// One record carrying `chunk` as content of type `kind`.
    fn seal(&mut self, kind: u8, chunk: &[u8]) -> Vec<u8> {
        if self.explicit_nonce {
            let sequence = self.sequence;
            let nonce = self.nonce();
            let additional = Self::tls12_additional(sequence, kind, chunk.len());
            let mut record = vec![kind];
            push_u16(&mut record, TLS1_2);
            push_u16(
                &mut record,
                (EXPLICIT_NONCE_LENGTH + chunk.len() + TAG_LENGTH) as u16,
            );
            record.extend_from_slice(&sequence.to_be_bytes());
            record.extend(self.gcm.seal(&nonce, &additional, chunk));
            return record;
        }
        let mut inner = chunk.to_vec();
        inner.push(kind);
        let mut record = vec![APPLICATION_DATA];
        push_u16(&mut record, TLS1_2);
        push_u16(&mut record, (inner.len() + TAG_LENGTH) as u16);
        let nonce = self.nonce();
        let sealed = self.gcm.seal(&nonce, &record, &inner);
        record.extend(sealed);
        record
    }

    /// The content type and plaintext of a sealed record, header included.
    fn open(&mut self, record: &[u8]) -> Result<(u8, Vec<u8>), TlsError> {
        let bad_mac = || failure(alert::BAD_RECORD_MAC, "bad record mac");
        if self.explicit_nonce {
            let body = &record[5..];
            if body.len() < EXPLICIT_NONCE_LENGTH + TAG_LENGTH {
                return Err(bad_mac());
            }
            let mut nonce = self.iv;
            nonce[4..].copy_from_slice(&body[..EXPLICIT_NONCE_LENGTH]);
            let additional = Self::tls12_additional(
                self.sequence,
                record[0],
                body.len() - EXPLICIT_NONCE_LENGTH - TAG_LENGTH,
            );
            self.sequence += 1;
            let plaintext = self
                .gcm
                .open(&nonce, &additional, &body[EXPLICIT_NONCE_LENGTH..])
                .ok_or_else(bad_mac)?;
            return Ok((record[0], plaintext));
        }
        let nonce = self.nonce();
        let mut inner = self
            .gcm
            .open(&nonce, &record[..5], &record[5..])
            .ok_or_else(bad_mac)?;
        while inner.last() == Some(&0) {
            inner.pop();
        }
        let Some(kind) = inner.pop() else {
            return Err(failure(alert::UNEXPECTED_MESSAGE, "unexpected message"));
        };
        Ok((kind, inner))
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
    /// Whether a server asks the client for a certificate, and whether it
    /// refuses a client that sends none.
    request_certificate: bool,
    require_certificate: bool,
    /// The context of the CertificateRequest a client was sent, which its
    /// Certificate answers with.
    certificate_request: Option<Vec<u8>>,
    /// The oldest and newest versions this side agrees to.
    min_version: u16,
    max_version: u16,
    /// The version agreed on, or zero before the hellos.
    version: u16,
    client_random: [u8; 32],
    server_random: [u8; 32],
    /// The stage a handshake paused for the chain decision goes on to.
    after_decision: Stage,
    /// TLS 1.2 keys that take effect once the peer's ChangeCipherSpec is
    /// read, and once this side sends its own.
    pending_reading: Option<Direction>,
    pending_writing: Option<Direction>,
    /// What a TLS 1.2 server read from the ClientHello and answers in kind.
    renegotiation_info_offered: bool,
    point_formats_offered: bool,
    signature_scheme: u16,
    /// The key share a TLS 1.2 server sent in its ServerKeyExchange.
    server_share: [u8; 32],
    /// The TLS 1.3 cipher suite agreed on.
    suite: u16,
    /// The TLS 1.3 cipher suites this side agrees to, in the order a
    /// client offers them.
    allowed_suites: Vec<u16>,
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
            request_certificate: false,
            require_certificate: false,
            certificate_request: None,
            min_version: TLS1_2,
            max_version: TLS1_3,
            version: 0,
            client_random: random_bytes(),
            server_random: random_bytes(),
            after_decision: Stage::Established,
            pending_reading: None,
            pending_writing: None,
            renegotiation_info_offered: false,
            point_formats_offered: false,
            signature_scheme: RSA_PSS_RSAE_SHA256,
            server_share: [0; 32],
            suite: TLS_AES_128_GCM_SHA256,
            allowed_suites: vec![
                TLS_AES_256_GCM_SHA384,
                TLS_CHACHA20_POLY1305_SHA256,
                TLS_AES_128_GCM_SHA256,
            ],
        }
    }

    /// Hold the connection to versions from `min` to `max`, where zero
    /// sets no limit.
    pub fn limit_versions(&mut self, min: u16, max: u16) {
        if min != 0 {
            self.min_version = min.clamp(TLS1_2, TLS1_3);
        }
        if max != 0 {
            self.max_version = max.clamp(TLS1_2, TLS1_3);
        }
    }

    /// The version agreed on, or zero before the hellos.
    pub fn version(&self) -> u16 {
        self.version
    }

    /// Hold TLS 1.3 to the cipher suites OpenSSL names, in that order,
    /// leaving out those not spoken here.
    pub fn limit_suites(&mut self, names: &[String]) {
        self.allowed_suites = names
            .iter()
            .filter_map(|name| match name.as_str() {
                "TLS_AES_128_GCM_SHA256" => Some(TLS_AES_128_GCM_SHA256),
                "TLS_AES_256_GCM_SHA384" => Some(TLS_AES_256_GCM_SHA384),
                "TLS_CHACHA20_POLY1305_SHA256" => Some(TLS_CHACHA20_POLY1305_SHA256),
                _ => None,
            })
            .collect();
    }

    /// The name OpenSSL gives the cipher suite agreed on.
    pub fn cipher_name(&self) -> &'static str {
        match (self.version, self.suite) {
            (TLS1_2, _) => "ECDHE-RSA-AES128-GCM-SHA256",
            (_, TLS_CHACHA20_POLY1305_SHA256) => "TLS_CHACHA20_POLY1305_SHA256",
            (_, TLS_AES_256_GCM_SHA384) => "TLS_AES_256_GCM_SHA384",
            _ => "TLS_AES_128_GCM_SHA256",
        }
    }

    /// The chain and key a client answers a CertificateRequest with.
    pub fn with_client_certificate(
        mut self,
        certificates: Vec<Vec<u8>>,
        private_key: RsaKey,
    ) -> Connection {
        self.certificates = certificates;
        self.private_key = Some(private_key);
        self
    }

    /// Have a server ask the client for a certificate, refusing a client
    /// that sends none when `required`.
    pub fn requesting_certificate(mut self, required: bool) -> Connection {
        self.request_certificate = true;
        self.require_certificate = required;
        self
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
        let speaks_1_3 = self.max_version >= TLS1_3;
        let speaks_1_2 = self.min_version <= TLS1_2;
        let mut body = Vec::new();
        push_u16(&mut body, TLS1_2);
        body.extend_from_slice(&self.client_random);
        body.extend(with_length(1, &random_bytes::<32>()));
        let mut suites = Vec::new();
        if speaks_1_3 {
            for suite in &self.allowed_suites {
                push_u16(&mut suites, *suite);
            }
        }
        if speaks_1_2 {
            push_u16(&mut suites, TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256);
        }
        body.extend(with_length(2, &suites));
        body.extend(with_length(1, &[0]));
        let mut listed = Vec::new();
        if let Some(name) = &self.server_name
            && name.parse::<std::net::IpAddr>().is_err()
        {
            let mut entry = vec![0u8];
            entry.extend(with_length(2, name.as_bytes()));
            listed.extend(extension(EXTENSION_SERVER_NAME, &with_length(2, &entry)));
        }
        if speaks_1_3 {
            let mut versions = TLS1_3.to_be_bytes().to_vec();
            if speaks_1_2 {
                push_u16(&mut versions, TLS1_2);
            }
            listed.extend(extension(
                EXTENSION_SUPPORTED_VERSIONS,
                &with_length(1, &versions),
            ));
        }
        if speaks_1_2 {
            listed.extend(extension(EXTENSION_EC_POINT_FORMATS, &with_length(1, &[0])));
            listed.extend(extension(
                EXTENSION_RENEGOTIATION_INFO,
                &with_length(1, &[]),
            ));
        }
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
        if speaks_1_3 {
            let mut share = Vec::new();
            push_u16(&mut share, GROUP_X25519);
            share.extend(with_length(2, &x25519::public_key(&self.secret_key)));
            listed.extend(extension(EXTENSION_KEY_SHARE, &with_length(2, &share)));
        }
        body.extend(with_length(2, &listed));
        handshake_message(CLIENT_HELLO, &body)
    }

    fn plain_record(&self, kind: u8, body: &[u8]) -> Vec<u8> {
        let mut written = Vec::new();
        for chunk in body.chunks(RECORD_LIMIT) {
            written.push(kind);
            push_u16(&mut written, TLS1_2);
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
            written.extend(direction.seal(kind, chunk));
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
        self.stage = self.after_decision;
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
            if kind == CHANGE_CIPHER_SPEC {
                // TLS 1.3 sends it only for middleboxes to read.
                if let Some(direction) = self.pending_reading.take() {
                    self.reading = Some(direction);
                }
                continue;
            }
            let (kind, body) = match self.reading.as_mut() {
                Some(direction) if direction.explicit_nonce || kind == APPLICATION_DATA => {
                    direction.open(&record)?
                }
                _ => (kind, record[5..].to_vec()),
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
                    // OpenSSL names an alert TLS 1.3 added with its version.
                    let version = if description == alert::CERTIFICATE_REQUIRED {
                        "tlsv13"
                    } else {
                        "tlsv1"
                    };
                    return Err(TlsError {
                        message: format!(
                            "{version} alert {} (SSL alert number {description})",
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
        if self.version == TLS1_2 {
            return self.handle_tls12_handshake(message, progress);
        }
        match (self.stage, kind) {
            (Stage::WaitServerHello, SERVER_HELLO) => self.read_server_hello(message, body),
            (Stage::WaitEncryptedExtensions, ENCRYPTED_EXTENSIONS) => {
                self.transcript.extend_from_slice(message);
                self.stage = Stage::WaitCertificate;
                Ok(())
            }
            (Stage::WaitCertificate, CERTIFICATE_REQUEST) => {
                let context = Reader::new(body).vector(1)?.to_vec();
                self.certificate_request = Some(context);
                self.transcript.extend_from_slice(message);
                Ok(())
            }
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
            (Stage::WaitClientCertificate, CERTIFICATE) => {
                self.read_client_certificate(message, body)
            }
            (Stage::WaitClientCertificateVerify, CERTIFICATE_VERIFY) => {
                self.read_certificate_verify(message, body)
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
        let mut version = reader.number(2)? as u16;
        self.server_random.copy_from_slice(reader.take(32)?);
        reader.vector(1)?;
        let suite = reader.number(2)? as u16;
        reader.take(1)?;
        let listed = if reader.done() {
            Vec::new()
        } else {
            extensions(reader.vector(2)?)?
        };
        let mut peer_share = None;
        for (kind, extension_body) in listed {
            match kind {
                EXTENSION_SUPPORTED_VERSIONS => {
                    version = Reader::new(extension_body).number(2)? as u16
                }
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
        if version < self.min_version || version > self.max_version {
            return Err(failure(alert::PROTOCOL_VERSION, "unsupported protocol"));
        }
        if version == TLS1_2 {
            return self.read_tls12_server_hello(message, suite);
        }
        if !self.allowed_suites.contains(&suite) {
            return Err(failure(alert::HANDSHAKE_FAILURE, "no shared cipher"));
        }
        self.suite = suite;
        self.version = version;
        let Some(peer_share) = peer_share else {
            return Err(failure(alert::HANDSHAKE_FAILURE, "no shared group"));
        };
        self.transcript.extend_from_slice(message);
        self.enter_handshake_keys(&peer_share);
        self.reading = Some(Direction::new(
            &self.server_handshake_secret.clone(),
            self.suite,
        ));
        self.writing = Some(Direction::new(
            &self.client_handshake_secret.clone(),
            self.suite,
        ));
        self.stage = Stage::WaitEncryptedExtensions;
        Ok(())
    }

    fn enter_handshake_keys(&mut self, peer_share: &[u8; 32]) {
        let shared = x25519::x25519(&self.secret_key, peer_share);
        let hash = suite_hash(self.suite);
        let early = crypto::hkdf_extract(hash, &[], &vec![0u8; hash.length()]);
        let derived = crypto::derive_secret(hash, &early, "derived", &[]);
        self.handshake_secret = crypto::hkdf_extract(hash, &derived, &shared);
        self.client_handshake_secret = crypto::derive_secret(
            suite_hash(self.suite),
            &self.handshake_secret,
            "c hs traffic",
            &self.transcript,
        );
        self.server_handshake_secret = crypto::derive_secret(
            suite_hash(self.suite),
            &self.handshake_secret,
            "s hs traffic",
            &self.transcript,
        );
        let derived = crypto::derive_secret(hash, &self.handshake_secret, "derived", &[]);
        self.master_secret = crypto::hkdf_extract(hash, &derived, &vec![0u8; hash.length()]);
    }

    /// The certificates of a Certificate message, leaf first.
    fn certificate_chain(body: &[u8]) -> Result<Vec<Vec<u8>>, TlsError> {
        let mut reader = Reader::new(body);
        reader.vector(1)?;
        let mut list = Reader::new(reader.vector(3)?);
        let mut chain = Vec::new();
        while !list.done() {
            chain.push(list.vector(3)?.to_vec());
            list.vector(2)?;
        }
        Ok(chain)
    }

    /// A server reads the client's chain. An empty one skips the
    /// CertificateVerify, and is refused when a certificate is required.
    fn read_client_certificate(&mut self, message: &[u8], body: &[u8]) -> Result<(), TlsError> {
        let chain = Self::certificate_chain(body)?;
        self.transcript.extend_from_slice(message);
        if chain.is_empty() {
            if self.require_certificate {
                return Err(failure(
                    alert::CERTIFICATE_REQUIRED,
                    "peer did not return a certificate",
                ));
            }
            self.stage = Stage::WaitClientFinished;
            return Ok(());
        }
        self.peer_certificates = chain;
        self.stage = Stage::WaitClientCertificateVerify;
        Ok(())
    }

    fn read_certificate(&mut self, message: &[u8], body: &[u8]) -> Result<(), TlsError> {
        let chain = Self::certificate_chain(body)?;
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
        content.extend_from_slice(&suite_hash(self.suite).digest(&self.transcript));
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
        // Each side checks what the other signed.
        let content = self.certificate_verify_content(self.is_client);
        let proved = match algorithm {
            RSA_PSS_RSAE_SHA256 => key.verify_pss(&content, signature),
            _ => false,
        };
        if !proved {
            return Err(failure(alert::DECRYPT_ERROR, "bad signature"));
        }
        self.transcript.extend_from_slice(message);
        self.stage = Stage::WaitCertificateDecision;
        self.after_decision = if self.is_client {
            Stage::WaitServerFinished
        } else {
            Stage::WaitClientFinished
        };
        Ok(())
    }

    fn finished_data(&self, secret: &[u8]) -> Vec<u8> {
        let hash = suite_hash(self.suite);
        let key = crypto::expand_label(hash, secret, "finished", &[], hash.length());
        hash.hmac(&key, &hash.digest(&self.transcript))
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
        let client_application = crypto::derive_secret(
            suite_hash(self.suite),
            &self.master_secret,
            "c ap traffic",
            &self.transcript,
        );
        let server_application = crypto::derive_secret(
            suite_hash(self.suite),
            &self.master_secret,
            "s ap traffic",
            &self.transcript,
        );
        let mut flight = Vec::new();
        if let Some(context) = self.certificate_request.take() {
            let certificate = self.certificate_message(&context);
            self.transcript.extend_from_slice(&certificate);
            flight.extend(certificate);
            if !self.certificates.is_empty() {
                let verify = self.certificate_verify_message(false)?;
                self.transcript.extend_from_slice(&verify);
                flight.extend(verify);
            }
        }
        let finished =
            handshake_message(FINISHED, &self.finished_data(&self.client_handshake_secret));
        self.transcript.extend_from_slice(&finished);
        flight.extend(finished);
        let records = self.sealed_records(HANDSHAKE, &flight);
        progress.outgoing.extend(records);
        self.reading = Some(Direction::new(&server_application, self.suite));
        self.writing = Some(Direction::new(&client_application, self.suite));
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
        let legacy_version = reader.number(2)? as u16;
        self.client_random.copy_from_slice(reader.take(32)?);
        let session = reader.vector(1)?.to_vec();
        let suites = reader.vector(2)?;
        reader.vector(1)?;
        let lists = |wanted: u16| suites.chunks(2).any(|suite| suite == wanted.to_be_bytes());
        let offered = suites
            .chunks(2)
            .map(|pair| u16::from_be_bytes([pair[0], pair[1]]))
            .find(|suite| self.allowed_suites.contains(suite));
        let offered_tls12_suite = lists(TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256);
        self.renegotiation_info_offered = lists(TLS_EMPTY_RENEGOTIATION_INFO_SCSV);
        let mut listed_versions: Option<Vec<u16>> = None;
        let mut share = None;
        let mut signs_with_pss = false;
        let mut signs_with_pkcs1 = false;
        let mut signature_algorithms_listed = false;
        let mut groups_name_x25519 = true;
        if !reader.done() {
            for (kind, extension_body) in extensions(reader.vector(2)?)? {
                match kind {
                    EXTENSION_SUPPORTED_VERSIONS => {
                        let versions = Reader::new(extension_body).vector(1)?;
                        listed_versions = Some(
                            versions
                                .chunks(2)
                                .filter(|version| version.len() == 2)
                                .map(|version| u16::from_be_bytes([version[0], version[1]]))
                                .collect(),
                        );
                    }
                    EXTENSION_SIGNATURE_ALGORITHMS => {
                        let algorithms = Reader::new(extension_body).vector(2)?;
                        signature_algorithms_listed = true;
                        signs_with_pss = algorithms
                            .chunks(2)
                            .any(|algorithm| algorithm == RSA_PSS_RSAE_SHA256.to_be_bytes());
                        signs_with_pkcs1 = algorithms
                            .chunks(2)
                            .any(|algorithm| algorithm == RSA_PKCS1_SHA256.to_be_bytes());
                    }
                    EXTENSION_SUPPORTED_GROUPS => {
                        let groups = Reader::new(extension_body).vector(2)?;
                        groups_name_x25519 = groups
                            .chunks(2)
                            .any(|group| group == GROUP_X25519.to_be_bytes());
                    }
                    EXTENSION_EC_POINT_FORMATS => self.point_formats_offered = true,
                    EXTENSION_RENEGOTIATION_INFO => self.renegotiation_info_offered = true,
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
        let allowed = |version: u16| version >= self.min_version && version <= self.max_version;
        let speaks_1_3 = allowed(TLS1_3)
            && listed_versions
                .as_ref()
                .is_some_and(|versions| versions.contains(&TLS1_3));
        let speaks_1_2 = allowed(TLS1_2)
            && match &listed_versions {
                Some(versions) => versions.contains(&TLS1_2),
                None => legacy_version >= TLS1_2,
            };
        if !speaks_1_3 && speaks_1_2 {
            if !offered_tls12_suite || !groups_name_x25519 {
                return Err(failure(alert::HANDSHAKE_FAILURE, "no shared cipher"));
            }
            self.signature_scheme = if signs_with_pss {
                RSA_PSS_RSAE_SHA256
            } else if signs_with_pkcs1 || !signature_algorithms_listed {
                RSA_PKCS1_SHA256
            } else {
                return Err(failure(
                    alert::HANDSHAKE_FAILURE,
                    "no suitable signature algorithm",
                ));
            };
            self.transcript.extend_from_slice(message);
            return self.send_tls12_server_flight(progress);
        }
        if !speaks_1_3 {
            return Err(failure(alert::PROTOCOL_VERSION, "unsupported protocol"));
        }
        self.version = TLS1_3;
        let Some(suite) = offered else {
            return Err(failure(alert::HANDSHAKE_FAILURE, "no shared cipher"));
        };
        self.suite = suite;
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
        self.reading = Some(Direction::new(
            &self.client_handshake_secret.clone(),
            self.suite,
        ));
        self.writing = Some(Direction::new(
            &self.server_handshake_secret.clone(),
            self.suite,
        ));
        let mut flight = handshake_message(ENCRYPTED_EXTENSIONS, &with_length(2, &[]));
        self.transcript.extend_from_slice(&flight);
        if self.request_certificate {
            let request = Self::certificate_request_message();
            self.transcript.extend_from_slice(&request);
            flight.extend_from_slice(&request);
        }
        let certificate = self.certificate_message(&[]);
        self.transcript.extend_from_slice(&certificate);
        flight.extend_from_slice(&certificate);
        let verify = self.certificate_verify_message(true)?;
        self.transcript.extend_from_slice(&verify);
        flight.extend_from_slice(&verify);
        let finished =
            handshake_message(FINISHED, &self.finished_data(&self.server_handshake_secret));
        self.transcript.extend_from_slice(&finished);
        flight.extend_from_slice(&finished);
        progress
            .outgoing
            .extend(self.sealed_records(HANDSHAKE, &flight));
        let server_application = crypto::derive_secret(
            suite_hash(self.suite),
            &self.master_secret,
            "s ap traffic",
            &self.transcript,
        );
        self.pending_client_secret = Some(crypto::derive_secret(
            suite_hash(self.suite),
            &self.master_secret,
            "c ap traffic",
            &self.transcript,
        ));
        self.writing = Some(Direction::new(&server_application, self.suite));
        self.stage = if self.request_certificate {
            Stage::WaitClientCertificate
        } else {
            Stage::WaitClientFinished
        };
        Ok(())
    }

    /// A CertificateRequest with an empty context, naming the one signature
    /// algorithm this side checks.
    fn certificate_request_message() -> Vec<u8> {
        let algorithms = with_length(2, &RSA_PSS_RSAE_SHA256.to_be_bytes());
        let mut body = with_length(1, &[]);
        body.extend(with_length(
            2,
            &extension(EXTENSION_SIGNATURE_ALGORITHMS, &algorithms),
        ));
        handshake_message(CERTIFICATE_REQUEST, &body)
    }

    fn server_hello(&self, session: &[u8]) -> Vec<u8> {
        let mut body = Vec::new();
        push_u16(&mut body, TLS1_2);
        body.extend_from_slice(&self.server_random);
        body.extend(with_length(1, session));
        push_u16(&mut body, self.suite);
        body.push(0);
        let mut listed = extension(EXTENSION_SUPPORTED_VERSIONS, &TLS1_3.to_be_bytes());
        let mut share = Vec::new();
        push_u16(&mut share, GROUP_X25519);
        share.extend(with_length(2, &x25519::public_key(&self.secret_key)));
        listed.extend(extension(EXTENSION_KEY_SHARE, &share));
        body.extend(with_length(2, &listed));
        handshake_message(SERVER_HELLO, &body)
    }

    fn certificate_message(&self, context: &[u8]) -> Vec<u8> {
        let mut list = Vec::new();
        for certificate in &self.certificates {
            list.extend(with_length(3, certificate));
            list.extend(with_length(2, &[]));
        }
        let mut body = with_length(1, context);
        body.extend(with_length(3, &list));
        handshake_message(CERTIFICATE, &body)
    }

    fn certificate_verify_message(&self, server: bool) -> Result<Vec<u8>, TlsError> {
        let Some(key) = &self.private_key else {
            return Err(failure(alert::INTERNAL_ERROR, "no private key"));
        };
        let content = self.certificate_verify_content(server);
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
        self.reading = Some(Direction::new(&client_application, self.suite));
        self.stage = Stage::Established;
        Ok(())
    }
}

mod tls12;

/// The server name a ClientHello asks for, read from the record that carries
/// it, so a server can choose its certificate before answering.
pub fn requested_name_in(record: &[u8]) -> Option<String> {
    if record.first() != Some(&HANDSHAKE) {
        return None;
    }
    let mut message = Reader::new(record.get(5..)?);
    if message.number(1).ok()? != CLIENT_HELLO as usize {
        return None;
    }
    let mut hello = Reader::new(message.vector(3).ok()?);
    hello.take(2 + 32).ok()?;
    hello.vector(1).ok()?;
    hello.vector(2).ok()?;
    hello.vector(1).ok()?;
    for (kind, body) in extensions(hello.vector(2).ok()?).ok()? {
        if kind != EXTENSION_SERVER_NAME {
            continue;
        }
        let mut names = Reader::new(Reader::new(body).vector(2).ok()?);
        if names.number(1).ok()? == 0 {
            return Some(String::from_utf8_lossy(names.vector(2).ok()?).into_owned());
        }
    }
    None
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
        alert::CERTIFICATE_REQUIRED => "certificate required",
        _ => "unknown",
    }
}
