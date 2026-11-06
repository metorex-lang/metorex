// `OpenSSL.__tls__`, which OpenSSL::SSL::SSLSocket runs its TLS connection
// through. The connection reads and writes no socket of its own: Ruby
// passes it the bytes read from the peer and sends the bytes it answers.

use super::*;
use crate::object::{OptionView, View};
use crate::tls::crypto::RsaKey;
use crate::tls::{Connection, Progress, TlsError};
use num_bigint::BigUint;

fn bytes_of(held: Option<&Object>) -> Vec<u8> {
    match held.view() {
        Some(View::String(text)) => text.bytes().to_vec(),
        _ => Vec::new(),
    }
}

fn binary(bytes: Vec<u8>) -> Object {
    Object::String(Rc::new(crate::object::StringValue::binary(bytes)))
}

fn unsigned(held: Option<&Object>) -> Option<BigUint> {
    held?.as_big_integer()?.to_biguint()
}

/// The bits of `OpenSSL::SSL::VERIFY_PEER` and `VERIFY_FAIL_IF_NO_PEER_CERT`.
const VERIFY_PEER: i64 = 1;
const VERIFY_FAIL_IF_NO_PEER_CERT: i64 = 2;

/// A chain of DER certificates, leaf first.
fn certificates(held: Option<&Object>) -> Vec<Vec<u8>> {
    match held.view() {
        Some(View::Array(listed)) => listed
            .borrow()
            .iter()
            .map(|der| bytes_of(Some(der)))
            .collect(),
        _ => Vec::new(),
    }
}

/// An RSA private key from its modulus and its public and private exponents.
fn rsa_key(parts: Option<&[Object]>) -> Option<RsaKey> {
    let [modulus, public_exponent, private_exponent] = parts? else {
        return None;
    };
    Some(RsaKey {
        modulus: unsigned(Some(modulus))?,
        public_exponent: unsigned(Some(public_exponent))?,
        private_exponent: Some(unsigned(Some(private_exponent))?),
    })
}

impl VirtualMachine {
    /// The state a connection is in, as the symbol SSLSocket reads it by.
    fn tls_state(connection: &Connection) -> Object {
        Object::symbol(if connection.waits_for_certificate_decision() {
            "decide"
        } else if connection.is_established() {
            "established"
        } else if connection.is_closed() {
            "closed"
        } else {
            "handshake"
        })
    }

    /// What a feed or a decision answers: the bytes to send, the
    /// application data read, whether the peer closed its side, the
    /// connection's state, and the failure's message when it failed. A
    /// failure found here also sends the peer the alert that names it.
    fn tls_progress(&mut self, number: usize, result: Result<Progress, TlsError>) -> Object {
        let Some(connection) = self.tls_connections.get_mut(&number) else {
            return Object::Nil;
        };
        match result {
            Ok(progress) => Object::array(vec![
                binary(progress.outgoing),
                binary(progress.plaintext),
                Object::Bool(progress.peer_closed),
                Self::tls_state(connection),
                Object::Nil,
            ]),
            Err(problem) => {
                let outgoing = if connection.is_closed() {
                    Vec::new()
                } else {
                    connection.alert_record(problem.alert)
                };
                Object::array(vec![
                    binary(outgoing),
                    binary(Vec::new()),
                    Object::Bool(false),
                    Object::symbol("failed"),
                    Object::string(problem.message),
                ])
            }
        }
    }

    /// `OpenSSL.__tls__(command, ...)`:
    ///
    /// * `:client, hostname[, certificates, n, e, d]` and `:server,
    ///   certificates, n, e, d, verify_mode` make a connection and answer its
    ///   number. The certificates are DER, leaf first, and n, e, and d are
    ///   the RSA key that proves them. A server asks for the client's
    ///   certificate when the verify mode has VERIFY_PEER.
    /// * `:hello_name, record` answers the server name the ClientHello in
    ///   the record asks for, or nil.
    /// * `:start, number` answers the bytes a client opens with.
    /// * `:feed, number, bytes` and `:decide, number, alert` answer what
    ///   `tls_progress` describes. The alert refuses the server's chain,
    ///   and nil trusts it.
    /// * `:write, number, bytes` answers the records carrying the bytes.
    /// * `:close, number` answers the alert that closes the connection.
    /// * `:peer_chain, number` and `:requested_name, number` answer what
    ///   the peer sent.
    /// * `:free, number` forgets the connection.
    pub(crate) fn tls_command(
        &mut self,
        arguments: &[Object],
        position: Position,
    ) -> Result<Object, MetorexError> {
        let command = match arguments.first().view() {
            Some(View::Symbol(name)) => name.as_str().to_string(),
            _ => String::new(),
        };
        if command == "client" || command == "server" {
            let connection = if command == "client" {
                let name = match arguments.get(1).view() {
                    Some(View::String(text)) => Some(text.as_str().to_string()),
                    _ => None,
                };
                let client = Connection::client(name);
                match rsa_key(arguments.get(3..6)) {
                    Some(key) => client.with_client_certificate(certificates(arguments.get(2)), key),
                    None => client,
                }
            } else {
                let Some(key) = rsa_key(arguments.get(2..5)) else {
                    return Err(simple_exception(
                        "OpenSSL::SSL::SSLError",
                        "no private key assigned",
                        position,
                    ));
                };
                let server = Connection::server(certificates(arguments.get(1)), key);
                let verify_mode = match arguments.get(5).view() {
                    Some(View::Int(mode)) => mode,
                    _ => 0,
                };
                if verify_mode & VERIFY_PEER != 0 {
                    server.requesting_certificate(verify_mode & VERIFY_FAIL_IF_NO_PEER_CERT != 0)
                } else {
                    server
                }
            };
            self.next_tls_connection += 1;
            let number = self.next_tls_connection;
            self.tls_connections.insert(number, connection);
            return Ok(Object::Int(number as i64));
        }
        if command == "hello_name" {
            return Ok(
                match crate::tls::connection::requested_name_in(&bytes_of(arguments.get(1))) {
                    Some(name) => Object::string(name),
                    None => Object::Nil,
                },
            );
        }
        let number = match arguments.get(1).view() {
            Some(View::Int(held)) => held as usize,
            _ => 0,
        };
        if command == "free" {
            self.tls_connections.remove(&number);
            return Ok(Object::Nil);
        }
        let Some(connection) = self.tls_connections.get_mut(&number) else {
            return Err(simple_exception(
                "OpenSSL::SSL::SSLError",
                "the connection is closed",
                position,
            ));
        };
        match command.as_str() {
            "start" => Ok(binary(connection.start())),
            "feed" => {
                let result = connection.feed(&bytes_of(arguments.get(2)));
                Ok(self.tls_progress(number, result))
            }
            "decide" => {
                let refused = match arguments.get(2).view() {
                    Some(View::Int(alert)) => Some(alert as u8),
                    _ => None,
                };
                let result = connection.decide_certificate(refused);
                Ok(self.tls_progress(number, result))
            }
            "write" => match connection.write(&bytes_of(arguments.get(2))) {
                Ok(records) => Ok(binary(records)),
                Err(problem) => Err(simple_exception(
                    "OpenSSL::SSL::SSLError",
                    &problem.message,
                    position,
                )),
            },
            "close" => Ok(binary(if connection.is_closed() {
                Vec::new()
            } else {
                connection.close()
            })),
            "peer_chain" => Ok(Object::array(
                connection
                    .peer_certificates
                    .iter()
                    .map(|der| binary(der.clone()))
                    .collect(),
            )),
            "requested_name" => Ok(match &connection.requested_name {
                Some(name) => Object::string(name.clone()),
                None => Object::Nil,
            }),
            _ => Ok(Object::Nil),
        }
    }
}
