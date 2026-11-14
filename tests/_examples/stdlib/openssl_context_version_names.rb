# SSLContext#min_version= and #max_version= take a version by name, such as
# :TLS1_2, as well as by number, and the handshake keeps to what they allow.
require "openssl"
require "socket"

key = OpenSSL::PKey::RSA.new(2048)
name = OpenSSL::X509::Name.new([%w[CN 127.0.0.1]])
cert = OpenSSL::X509::Certificate.new
cert.version = 2
cert.serial = 1
cert.issuer = name
cert.subject = name
cert.not_before = Time.now
cert.not_after = Time.now + 3600
cert.public_key = key.public_key
cert.sign(key, OpenSSL::Digest::SHA256.new)

server_context = OpenSSL::SSL::SSLContext.new
server_context.cert = cert
server_context.key = key
server_context.min_version = :TLS1_3
server_context.max_version = "TLS1_3"

server = TCPServer.new("127.0.0.1", 0)
client = TCPSocket.open("127.0.0.1", server.local_address.ip_port)
peer = server.accept
ssl_peer = OpenSSL::SSL::SSLSocket.new(peer, server_context)
ssl_client = OpenSSL::SSL::SSLSocket.new(client)
handshake = Thread.new { ssl_client.connect }
ssl_peer.accept
handshake.join
p ssl_peer.ssl_version

[:TLS9, 1.5].each do |version|
  OpenSSL::SSL::SSLContext.new.min_version = version
rescue ArgumentError, TypeError => error
  p [error.class, error.message]
end
