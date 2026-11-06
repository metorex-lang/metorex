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
listener = TCPServer.new("127.0.0.1", 0)

server = Thread.new do
  connection = OpenSSL::SSL::SSLSocket.new(listener.accept, context)
  connection.accept
  # The request the client wrote right after the handshake is still waiting
  # on the socket itself.
  ready = connection.to_io.wait_readable(5)
  line = connection.gets
  connection.write("#{ready ? "ready" : "not ready"}: #{line}")
  connection.close
end

client_context = OpenSSL::SSL::SSLContext.new
client_context.verify_mode = OpenSSL::SSL::VERIFY_NONE
LOGGED = []
module LoggedReads
  def readpartial(*) = super.tap { |held| LOGGED << held }
  def write(*texts) = super.tap { LOGGED << texts.join }
end
raw = TCPSocket.new("127.0.0.1", listener.addr[1])
raw.extend(LoggedReads)
client = OpenSSL::SSL::SSLSocket.new(raw, client_context)
client.connect
client.write("GET / HTTP/1.0\n")
p client.gets
server.join
p LOGGED
