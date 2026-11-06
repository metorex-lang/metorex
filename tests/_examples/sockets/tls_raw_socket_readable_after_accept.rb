# The plain socket under a server's TLS socket is readable once the client
# has written, even when the client wrote right after the handshake.
require "openssl"
require "socket"

fixtures = File.join(__dir__, "tls_fixtures")
context = OpenSSL::SSL::SSLContext.new
context.key = File.open(File.join(fixtures, "server.key")) { |file| OpenSSL::PKey::RSA.new(file) }
context.cert = File.open(File.join(fixtures, "server.crt")) { |file| OpenSSL::X509::Certificate.new(file) }
listener = TCPServer.new("127.0.0.1", 0)
server = Thread.start do
  secure = OpenSSL::SSL::SSLSocket.new(listener.accept, context)
  secure.sync_close = true
  secure.accept
  sleep 0.2
  readable = !secure.to_io.wait_readable(2).nil?
  line = secure.gets
  secure.close
  [readable, line]
end

client_context = OpenSSL::SSL::SSLContext.new
client_context.set_params(verify_mode: OpenSSL::SSL::VERIFY_NONE)
client = OpenSSL::SSL::SSLSocket.new(Socket.tcp("127.0.0.1", listener.addr[1]), client_context)
client.sync_close = true
client.connect
client.write("GET / HTTP/1.1\r\n")
p server.value
client.close
listener.close
