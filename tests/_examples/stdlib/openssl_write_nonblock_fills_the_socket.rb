# SSLSocket#write_nonblock raises SSLErrorWaitWritable once the socket can
# take no more, rather than waiting, and select then reports the socket as
# not writable.
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

context = OpenSSL::SSL::SSLContext.new
context.cert = cert
context.key = key

server = TCPServer.new("127.0.0.1", 0)
client = TCPSocket.new("127.0.0.1", server.local_address.ip_port)
peer = server.accept
ssl_peer = OpenSSL::SSL::SSLSocket.new(peer, context)
ssl_client = OpenSSL::SSL::SSLSocket.new(client)
handshake = Thread.new { ssl_client.connect }
ssl_peer.accept
handshake.join

writes = 0
begin
  loop do
    ssl_client.write_nonblock("X" * 1024)
    writes += 1
  end
rescue OpenSSL::SSL::SSLErrorWaitWritable => error
  p error.class.ancestors.include?(IO::WaitWritable), writes > 0
end
p select([], [ssl_client], [], 0)
p ssl_client.write_nonblock("X" * 1024, exception: false)
