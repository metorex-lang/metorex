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
    /// * `:client, hostname` and `:server, certificates, n, e, d` make a
    ///   connection and answer its number. The certificates are DER, leaf
    ///   first, and n, e, and d are the server's RSA key.
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
                Connection::client(name)
            } else {
                let certificates = match arguments.get(1).view() {
                    Some(View::Array(held)) => held
                        .borrow()
                        .iter()
                        .map(|der| bytes_of(Some(der)))
                        .collect(),
                    _ => Vec::new(),
                };
                let (Some(modulus), Some(public_exponent), Some(private_exponent)) = (
                    unsigned(arguments.get(2)),
                    unsigned(arguments.get(3)),
                    unsigned(arguments.get(4)),
                ) else {
                    return Err(simple_exception(
                        "OpenSSL::SSL::SSLError",
                        "no private key assigned",
                        position,
                    ));
                };
                let key = RsaKey {
                    modulus,
                    public_exponent,
                    private_exponent: Some(private_exponent),
                };
                Connection::server(certificates, key)
            };
            self.next_tls_connection += 1;
            let number = self.next_tls_connection;
            self.tls_connections.insert(number, connection);
            return Ok(Object::Int(number as i64));
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
