// The TLS 1.2 handshake (RFC 5246) with ECDHE over X25519 (RFC 8422) and
// AES-128-GCM records (RFC 5288). The hellos are read in connection.rs,
// which hands over here once both sides agree to TLS 1.2.

use super::*;

/// The bytes of the master secret and of each Finished's verify_data.
const MASTER_SECRET_LENGTH: usize = 48;
const VERIFY_DATA_LENGTH: usize = 12;
/// A key block holds two 16-byte keys and two 4-byte salts.
const KEY_LENGTH: usize = 16;
const SALT_LENGTH: usize = 4;
/// The certificate_type a CertificateRequest asks for RSA keys by.
const RSA_SIGN: u8 = 1;

impl Connection {
    pub(super) fn handle_tls12_handshake(
        &mut self,
        message: &[u8],
        progress: &mut Progress,
    ) -> Result<(), TlsError> {
        let kind = message[0];
        let body = &message[4..];
        match (self.stage, kind) {
            (Stage::WaitCertificate, CERTIFICATE) => {
                let chain = Self::tls12_certificate_chain(body)?;
                if chain.is_empty() {
                    return Err(failure(
                        alert::BAD_CERTIFICATE,
                        "peer did not return a certificate",
                    ));
                }
                self.peer_certificates = chain;
                self.transcript.extend_from_slice(message);
                self.stage = Stage::WaitServerKeyExchange;
                Ok(())
            }
            (Stage::WaitServerKeyExchange, SERVER_KEY_EXCHANGE) => {
                self.read_server_key_exchange(message, body)
            }
            (Stage::WaitServerHelloDone, CERTIFICATE_REQUEST) => {
                self.certificate_request = Some(Vec::new());
                self.transcript.extend_from_slice(message);
                Ok(())
            }
            (Stage::WaitServerHelloDone, SERVER_HELLO_DONE) => {
                self.transcript.extend_from_slice(message);
                self.send_tls12_client_flight(progress)
            }
            (Stage::WaitServerFinished, FINISHED) => {
                self.check_tls12_finished(message, "server finished")?;
                self.stage = Stage::Established;
                Ok(())
            }
            (Stage::WaitClientCertificate, CERTIFICATE) => {
                let chain = Self::tls12_certificate_chain(body)?;
                if chain.is_empty() && self.require_certificate {
                    return Err(failure(
                        alert::HANDSHAKE_FAILURE,
                        "peer did not return a certificate",
                    ));
                }
                self.peer_certificates = chain;
                self.transcript.extend_from_slice(message);
                self.stage = Stage::WaitClientKeyExchange;
                Ok(())
            }
            (Stage::WaitClientKeyExchange, CLIENT_KEY_EXCHANGE) => {
                self.read_client_key_exchange(message, body)
            }
            (Stage::WaitClientCertificateVerify, CERTIFICATE_VERIFY) => {
                let mut reader = Reader::new(body);
                let scheme = reader.number(2)? as u16;
                let signature = reader.vector(2)?;
                let signed = self.transcript.clone();
                self.check_signature(scheme, &signed, signature)?;
                self.transcript.extend_from_slice(message);
                self.stage = Stage::WaitCertificateDecision;
                self.after_decision = Stage::WaitClientFinished;
                Ok(())
            }
            (Stage::WaitClientFinished, FINISHED) => {
                self.check_tls12_finished(message, "client finished")?;
                progress
                    .outgoing
                    .extend(self.plain_record(CHANGE_CIPHER_SPEC, &[1]));
                self.writing = self.pending_writing.take();
                let finished = self.tls12_finished("server finished");
                self.transcript.extend_from_slice(&finished);
                progress
                    .outgoing
                    .extend(self.sealed_records(HANDSHAKE, &finished));
                self.stage = Stage::Established;
                Ok(())
            }
            _ => Err(failure(alert::UNEXPECTED_MESSAGE, "unexpected message")),
        }
    }

    /// A client takes the server's TLS 1.2 answer, which a server able to
    /// speak TLS 1.3 marks in its random when the client offered TLS 1.3.
    pub(super) fn read_tls12_server_hello(
        &mut self,
        message: &[u8],
        suite: u16,
    ) -> Result<(), TlsError> {
        if suite != TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256 {
            return Err(failure(alert::HANDSHAKE_FAILURE, "no shared cipher"));
        }
        if self.max_version >= TLS1_3 && &self.server_random[24..] == DOWNGRADE_TO_TLS1_2 {
            return Err(failure(alert::ILLEGAL_PARAMETER, "inappropriate fallback"));
        }
        self.version = TLS1_2;
        self.transcript.extend_from_slice(message);
        self.stage = Stage::WaitCertificate;
        Ok(())
    }

    /// The certificates of a TLS 1.2 Certificate message, leaf first.
    fn tls12_certificate_chain(body: &[u8]) -> Result<Vec<Vec<u8>>, TlsError> {
        let mut list = Reader::new(Reader::new(body).vector(3)?);
        let mut chain = Vec::new();
        while !list.done() {
            chain.push(list.vector(3)?.to_vec());
        }
        Ok(chain)
    }

    fn tls12_certificate_message(&self) -> Vec<u8> {
        let list: Vec<u8> = self
            .certificates
            .iter()
            .flat_map(|certificate| with_length(3, certificate))
            .collect();
        handshake_message(CERTIFICATE, &with_length(3, &list))
    }

    /// Check what the peer signed with the key of the leaf it sent.
    fn check_signature(
        &self,
        scheme: u16,
        signed: &[u8],
        signature: &[u8],
    ) -> Result<(), TlsError> {
        let Some(key) = self
            .peer_certificates
            .first()
            .and_then(|leaf| super::super::certificate::rsa_public_key(leaf))
        else {
            return Err(failure(
                alert::BAD_CERTIFICATE,
                "unsupported certificate key",
            ));
        };
        let proved = match scheme {
            RSA_PSS_RSAE_SHA256 => key.verify_pss(signed, signature),
            RSA_PKCS1_SHA256 => {
                key.verify_pkcs1(&crypto::SHA256_DIGEST_INFO, &sha256(signed), signature)
            }
            _ => false,
        };
        if proved {
            Ok(())
        } else {
            Err(failure(alert::DECRYPT_ERROR, "bad signature"))
        }
    }

    /// The scheme and signature over `signed`, made with this side's key.
    fn tls12_signature(&self, scheme: u16, signed: &[u8]) -> Result<Vec<u8>, TlsError> {
        let signature = self
            .private_key
            .as_ref()
            .and_then(|key| match scheme {
                RSA_PSS_RSAE_SHA256 => key.sign_pss(signed, &random_bytes::<HASH_LENGTH>()),
                _ => key.sign_pkcs1(&crypto::SHA256_DIGEST_INFO, &sha256(signed)),
            })
            .ok_or_else(|| failure(alert::INTERNAL_ERROR, "the key could not sign"))?;
        let mut written = scheme.to_be_bytes().to_vec();
        written.extend(with_length(2, &signature));
        Ok(written)
    }

    /// The ECDHE parameters of a ServerKeyExchange: the curve and the
    /// server's public key.
    fn key_exchange_parameters(&self) -> Vec<u8> {
        let mut parameters = vec![NAMED_CURVE];
        push_u16(&mut parameters, GROUP_X25519);
        parameters.extend(with_length(1, &x25519::public_key(&self.secret_key)));
        parameters
    }

    /// What a ServerKeyExchange signs: both randoms, then the parameters.
    fn key_exchange_signed(&self, parameters: &[u8]) -> Vec<u8> {
        let mut signed = self.client_random.to_vec();
        signed.extend_from_slice(&self.server_random);
        signed.extend_from_slice(parameters);
        signed
    }

    fn read_server_key_exchange(&mut self, message: &[u8], body: &[u8]) -> Result<(), TlsError> {
        let mut reader = Reader::new(body);
        let curve_type = reader.number(1)? as u8;
        let group = reader.number(2)? as u16;
        let point = reader.vector(1)?;
        if curve_type != NAMED_CURVE || group != GROUP_X25519 || point.len() != 32 {
            return Err(failure(alert::ILLEGAL_PARAMETER, "wrong curve"));
        }
        let parameters = &body[..reader.at];
        let scheme = reader.number(2)? as u16;
        let signature = reader.vector(2)?;
        self.check_signature(scheme, &self.key_exchange_signed(parameters), signature)?;
        self.server_share.copy_from_slice(point);
        self.transcript.extend_from_slice(message);
        self.stage = Stage::WaitCertificateDecision;
        self.after_decision = Stage::WaitServerHelloDone;
        Ok(())
    }

    /// Derive the master secret from the X25519 secret, then the keys each
    /// side writes with. Answers the client's and the server's directions.
    fn enter_tls12_keys(&mut self, peer_share: &[u8; 32]) -> (Direction, Direction) {
        let premaster = x25519::x25519(&self.secret_key, peer_share);
        let mut randoms = self.client_random.to_vec();
        randoms.extend_from_slice(&self.server_random);
        self.master_secret =
            crypto::prf(&premaster, "master secret", &randoms, MASTER_SECRET_LENGTH);
        let mut seed = self.server_random.to_vec();
        seed.extend_from_slice(&self.client_random);
        let block = crypto::prf(
            &self.master_secret,
            "key expansion",
            &seed,
            2 * (KEY_LENGTH + SALT_LENGTH),
        );
        let (keys, salts) = block.split_at(2 * KEY_LENGTH);
        (
            Direction::tls12(&keys[..KEY_LENGTH], &salts[..SALT_LENGTH]),
            Direction::tls12(&keys[KEY_LENGTH..], &salts[SALT_LENGTH..]),
        )
    }

    fn tls12_finished(&self, label: &str) -> Vec<u8> {
        let verify_data = crypto::prf(
            &self.master_secret,
            label,
            &sha256(&self.transcript),
            VERIFY_DATA_LENGTH,
        );
        handshake_message(FINISHED, &verify_data)
    }

    /// Check the peer's Finished, which must come after its ChangeCipherSpec.
    fn check_tls12_finished(&mut self, message: &[u8], label: &str) -> Result<(), TlsError> {
        if self.pending_reading.is_some() {
            return Err(failure(alert::UNEXPECTED_MESSAGE, "unexpected message"));
        }
        if message != self.tls12_finished(label) {
            return Err(failure(alert::DECRYPT_ERROR, "digest check failed"));
        }
        self.transcript.extend_from_slice(message);
        Ok(())
    }

    /// After ServerHelloDone a client sends its certificate when asked, its
    /// key share, its proof of the certificate, then ChangeCipherSpec and
    /// its Finished.
    fn send_tls12_client_flight(&mut self, progress: &mut Progress) -> Result<(), TlsError> {
        let mut flight = Vec::new();
        let asked = self.certificate_request.take().is_some();
        if asked {
            let certificate = self.tls12_certificate_message();
            self.transcript.extend_from_slice(&certificate);
            flight.extend(certificate);
        }
        let exchange = handshake_message(
            CLIENT_KEY_EXCHANGE,
            &with_length(1, &x25519::public_key(&self.secret_key)),
        );
        self.transcript.extend_from_slice(&exchange);
        flight.extend(exchange);
        if asked && !self.certificates.is_empty() {
            let signed = self.tls12_signature(RSA_PSS_RSAE_SHA256, &self.transcript)?;
            let verify = handshake_message(CERTIFICATE_VERIFY, &signed);
            self.transcript.extend_from_slice(&verify);
            flight.extend(verify);
        }
        let server_share = self.server_share;
        let (client, server) = self.enter_tls12_keys(&server_share);
        progress
            .outgoing
            .extend(self.plain_record(HANDSHAKE, &flight));
        progress
            .outgoing
            .extend(self.plain_record(CHANGE_CIPHER_SPEC, &[1]));
        self.writing = Some(client);
        let finished = self.tls12_finished("client finished");
        self.transcript.extend_from_slice(&finished);
        progress
            .outgoing
            .extend(self.sealed_records(HANDSHAKE, &finished));
        self.pending_reading = Some(server);
        self.stage = Stage::WaitServerFinished;
        Ok(())
    }

    /// A server answers a TLS 1.2 ClientHello with its hello, certificate,
    /// key share, any certificate request, and ServerHelloDone.
    pub(super) fn send_tls12_server_flight(
        &mut self,
        progress: &mut Progress,
    ) -> Result<(), TlsError> {
        self.version = TLS1_2;
        if self.max_version >= TLS1_3 {
            self.server_random[24..].copy_from_slice(DOWNGRADE_TO_TLS1_2);
        }
        let mut hello = Vec::new();
        push_u16(&mut hello, TLS1_2);
        hello.extend_from_slice(&self.server_random);
        hello.extend(with_length(1, &[]));
        push_u16(&mut hello, TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256);
        hello.push(0);
        let mut listed = Vec::new();
        if self.renegotiation_info_offered {
            listed.extend(extension(
                EXTENSION_RENEGOTIATION_INFO,
                &with_length(1, &[]),
            ));
        }
        if self.point_formats_offered {
            listed.extend(extension(EXTENSION_EC_POINT_FORMATS, &with_length(1, &[0])));
        }
        if !listed.is_empty() {
            hello.extend(with_length(2, &listed));
        }
        let mut flight = handshake_message(SERVER_HELLO, &hello);
        flight.extend(self.tls12_certificate_message());
        let parameters = self.key_exchange_parameters();
        let mut exchange = parameters.clone();
        exchange.extend(self.tls12_signature(
            self.signature_scheme,
            &self.key_exchange_signed(&parameters),
        )?);
        flight.extend(handshake_message(SERVER_KEY_EXCHANGE, &exchange));
        if self.request_certificate {
            let mut request = with_length(1, &[RSA_SIGN]);
            let mut algorithms = RSA_PSS_RSAE_SHA256.to_be_bytes().to_vec();
            push_u16(&mut algorithms, RSA_PKCS1_SHA256);
            request.extend(with_length(2, &algorithms));
            request.extend(with_length(2, &[]));
            flight.extend(handshake_message(CERTIFICATE_REQUEST, &request));
        }
        flight.extend(handshake_message(SERVER_HELLO_DONE, &[]));
        self.transcript.extend_from_slice(&flight);
        progress
            .outgoing
            .extend(self.plain_record(HANDSHAKE, &flight));
        self.stage = if self.request_certificate {
            Stage::WaitClientCertificate
        } else {
            Stage::WaitClientKeyExchange
        };
        Ok(())
    }

    fn read_client_key_exchange(&mut self, message: &[u8], body: &[u8]) -> Result<(), TlsError> {
        let point = Reader::new(body).vector(1)?;
        if point.len() != 32 {
            return Err(failure(alert::DECODE_ERROR, "bad ecpoint"));
        }
        let mut share = [0u8; 32];
        share.copy_from_slice(point);
        self.transcript.extend_from_slice(message);
        let (client, server) = self.enter_tls12_keys(&share);
        self.pending_reading = Some(client);
        self.pending_writing = Some(server);
        self.stage = if self.peer_certificates.is_empty() {
            Stage::WaitClientFinished
        } else {
            Stage::WaitClientCertificateVerify
        };
        Ok(())
    }
}
