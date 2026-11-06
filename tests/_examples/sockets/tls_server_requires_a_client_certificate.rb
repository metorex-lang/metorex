# A server that requires a client certificate refuses a client without one,
# and both ends raise SSLError.
require "openssl"
require "socket"

fixtures = File.join(__dir__, "tls_fixtures")
context = OpenSSL::SSL::SSLContext.new
context.key = File.open(File.join(fixtures, "server.key")) { |file| OpenSSL::PKey::RSA.new(file) }
context.cert = File.open(File.join(fixtures, "server.crt")) { |file| OpenSSL::X509::Certificate.new(file) }
context.ca_file = File.join(fixtures, "cacert.pem")
context.verify_mode = OpenSSL::SSL::VERIFY_PEER | OpenSSL::SSL::VERIFY_FAIL_IF_NO_PEER_CERT
listener = TCPServer.new("127.0.0.1", 0)
server = Thread.start do
  secure = OpenSSL::SSL::SSLSocket.new(listener.accept, context)
  secure.sync_close = true
  begin
    secure.accept
    :accepted
  rescue OpenSSL::SSL::SSLError => error
    error.class
  ensure
    secure.close
  end
end

client_context = OpenSSL::SSL::SSLContext.new
client_context.set_params(verify_mode: OpenSSL::SSL::VERIFY_NONE)
client = OpenSSL::SSL::SSLSocket.new(Socket.tcp("127.0.0.1", listener.addr[1]), client_context)
client.sync_close = true
outcome = begin
  client.connect
  client.write("GET / HTTP/1.1\r\n\r\n")
  client.read
rescue OpenSSL::SSL::SSLError, SystemCallError, IOError => error
  error.class.ancestors.include?(OpenSSL::SSL::SSLError) ? OpenSSL::SSL::SSLError : error.class
end
p outcome
p server.value
client.close
listener.close

# A client presenting a certificate the server trusts is let in, and the
# server reads that certificate.
listener = TCPServer.new("127.0.0.1", 0)
server = Thread.start do
  secure = OpenSSL::SSL::SSLSocket.new(listener.accept, context)
  secure.sync_close = true
  secure.accept
  secure.write("hello #{secure.peer_cert.subject}\n")
  secure.close
  secure.peer_cert.serial == context.cert.serial
end
client_context = OpenSSL::SSL::SSLContext.new
client_context.set_params(verify_mode: OpenSSL::SSL::VERIFY_NONE)
client_context.cert = context.cert
client_context.key = context.key
client = OpenSSL::SSL::SSLSocket.new(Socket.tcp("127.0.0.1", listener.addr[1]), client_context)
client.sync_close = true
client.connect
p client.gets
p server.value
client.close
listener.close
