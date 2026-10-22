require "openssl"
require "socket"

fixtures = File.join(__dir__, "tls_fixtures")
certificate = OpenSSL::X509::Certificate.new(File.read(File.join(fixtures, "server.crt")))
authority = OpenSSL::X509::Certificate.new(File.read(File.join(fixtures, "cacert.pem")))
key = OpenSSL::PKey::RSA.new(File.read(File.join(fixtures, "server.key")))

p certificate.subject.to_s, certificate.issuer.to_s
p certificate.serial.to_i, certificate.version, certificate.signature_algorithm
p certificate.not_before, certificate.not_after
p certificate.verify(authority.public_key), certificate.public_key.n == key.n
p certificate.to_der == OpenSSL::X509::Certificate.new(certificate.to_der).to_der

server_context = OpenSSL::SSL::SSLContext.new
server_context.cert = certificate
server_context.key = key

# Each connection is served by a thread that echoes one line, or answers
# the error the handshake ended in.
def serve(context)
  listener = TCPServer.new("127.0.0.1", 0)
  secure = OpenSSL::SSL::SSLServer.new(listener, context)
  thread = Thread.new do
    socket = secure.accept
    line = socket.gets
    socket.write("echo: #{line}")
    socket.close
    :served
  rescue OpenSSL::SSL::SSLError => error
    [error.class, error.message.sub(/peeraddr=\S+ /, "")]
  ensure
    listener.close
  end
  [listener.addr[1], thread]
end

def client_context(**params)
  context = OpenSSL::SSL::SSLContext.new
  context.set_params(params)
  context.ciphersuites = "TLS_AES_128_GCM_SHA256"
  context
end

port, thread = serve(server_context)
socket = OpenSSL::SSL::SSLSocket.new(TCPSocket.new("127.0.0.1", port), client_context(ca_file: File.join(fixtures, "cacert.pem")))
socket.hostname = "localhost"
socket.sync_close = true
socket.connect
p socket.post_connection_check("localhost")
p socket.ssl_version, socket.cipher[0], socket.verify_result
p socket.peer_cert.subject.to_s, socket.peer_cert_chain.size
socket.puts "a001 NOOP"
p socket.gets, socket.read
socket.close
p thread.value

begin
  socket.post_connection_check("127.0.0.1")
rescue OpenSSL::SSL::SSLError => error
  p error.message
end

port, thread = serve(server_context)
socket = OpenSSL::SSL::SSLSocket.new(TCPSocket.new("127.0.0.1", port), client_context)
socket.hostname = "localhost"
begin
  socket.connect
rescue OpenSSL::SSL::SSLError => error
  p error.message.sub(/peeraddr=\S+ /, "")
end
socket.io.close
p thread.value

port, thread = serve(server_context)
socket = OpenSSL::SSL::SSLSocket.new(TCPSocket.new("127.0.0.1", port), client_context(verify_mode: OpenSSL::SSL::VERIFY_NONE))
socket.connect
p socket.verify_result
socket.write("plain\n")
p socket.readpartial(100)
socket.close
p thread.value
