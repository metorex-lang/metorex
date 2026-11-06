require "openssl"
require "socket"

key = OpenSSL::PKey::RSA.new(2048)
name = OpenSSL::X509::Name.parse("/CN=localhost")
certificate = OpenSSL::X509::Certificate.new
certificate.version = 2
certificate.serial = 1
certificate.subject = name
certificate.issuer = name
certificate.public_key = key.public_key
certificate.not_before = Time.now - 60
certificate.not_after = Time.now + 3600
certificate.sign(key, "SHA256")

context = OpenSSL::SSL::SSLContext.new
context.cert = certificate
context.key = key
context.ssl_version = :TLSv1
listener = TCPServer.new("127.0.0.1", 0)

server = Thread.new do
  connection = OpenSSL::SSL::SSLSocket.new(listener.accept, context)
  begin
    connection.accept
  rescue OpenSSL::SSL::SSLError => error
    error.message[/state=error: .*/]
  end
end

client_context = OpenSSL::SSL::SSLContext.new
client_context.verify_mode = OpenSSL::SSL::VERIFY_NONE
client = OpenSSL::SSL::SSLSocket.new(TCPSocket.new("127.0.0.1", listener.addr[1]), client_context)
begin
  client.connect
rescue OpenSSL::SSL::SSLError => error
  p error.message[/tlsv1 alert protocol version/]
end
p server.value
