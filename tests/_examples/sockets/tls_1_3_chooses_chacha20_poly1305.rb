require "openssl"
require "socket"

key = OpenSSL::PKey::RSA.new(2048)
name = OpenSSL::X509::Name.parse("/CN=localhost")
certificate = OpenSSL::X509::Certificate.new
certificate.version = 2
certificate.serial = 1
certificate.subject = name
certificate.issuer = name
certificate.public_key = key
certificate.not_before = Time.now - 60
certificate.not_after = Time.now + 3600
certificate.sign(key, "SHA256")

def exchange(listener, certificate, key, server_suites, client_suites)
  serving = Thread.new do
    context = OpenSSL::SSL::SSLContext.new
    context.cert = certificate
    context.key = key
    context.ciphersuites = server_suites if server_suites
    connection = OpenSSL::SSL::SSLSocket.new(listener.accept, context)
    connection.accept
    connection.write(connection.read(4).upcase)
    connection.close
  end
  context = OpenSSL::SSL::SSLContext.new
  context.ciphersuites = client_suites if client_suites
  socket = OpenSSL::SSL::SSLSocket.new(TCPSocket.new("127.0.0.1", listener.addr[1]), context)
  socket.connect
  socket.write("ping")
  p [socket.read, socket.cipher]
  serving.join
end

listener = TCPServer.new("127.0.0.1", 0)
exchange(listener, certificate, key, nil, nil)
exchange(listener, certificate, key, "TLS_CHACHA20_POLY1305_SHA256", nil)
exchange(listener, certificate, key, nil, "TLS_CHACHA20_POLY1305_SHA256")
exchange(listener, certificate, key, nil, "TLS_AES_128_GCM_SHA256:TLS_CHACHA20_POLY1305_SHA256")
context = OpenSSL::SSL::SSLContext.new
context.ciphersuites = ["TLS_AES_128_GCM_SHA256", "TLS_CHACHA20_POLY1305_SHA256"]
p context.ciphers.first(2).map(&:first)
begin
  context.ciphersuites = "NOPE"
rescue OpenSSL::SSL::SSLError => error
  p error.message
end
