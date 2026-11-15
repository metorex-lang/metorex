require "openssl"
require "socket"
require "tempfile"

def certificate(subject, issuer, key, signer, serial, authority)
  made = OpenSSL::X509::Certificate.new
  made.version = 2
  made.serial = serial
  made.subject = OpenSSL::X509::Name.parse(subject)
  made.issuer = OpenSSL::X509::Name.parse(issuer)
  made.public_key = key
  made.not_before = Time.now - 60
  made.not_after = Time.now + 3600
  if authority
    extensions = OpenSSL::X509::ExtensionFactory.new
    made.add_extension(extensions.create_extension("basicConstraints", "CA:TRUE", true))
  end
  made.sign(signer, "SHA256")
end

authority_key = OpenSSL::PKey::RSA.new(2048)
client_key = OpenSSL::PKey::RSA.new(2048)
server_key = OpenSSL::PKey::RSA.new(2048)
authority = certificate("/CN=Unknown CA", "/CN=Unknown CA", authority_key, authority_key, 1, true)
client = certificate("/CN=client", "/CN=Unknown CA", client_key, authority_key, 2, false)
server = certificate("/CN=server", "/CN=server", server_key, server_key, 3, false)

authority_file = Tempfile.new(["authority", ".pem"])
authority_file.write(authority.to_pem)
authority_file.flush

listener = TCPServer.new("127.0.0.1", 0)
received = []
serving = Thread.new do
  context = OpenSSL::SSL::SSLContext.new
  context.cert = server
  context.key = server_key
  context.verify_mode = OpenSSL::SSL::VERIFY_PEER | OpenSSL::SSL::VERIFY_FAIL_IF_NO_PEER_CERT
  context.verify_callback = lambda do |_ok, store_context|
    received << store_context.current_cert.subject.to_s
    true
  end
  connection = OpenSSL::SSL::SSLSocket.new(listener.accept, context)
  connection.accept
  connection.write("ok")
  connection.close
end

context = OpenSSL::SSL::SSLContext.new
context.cert = client
context.key = client_key
context.ca_file = authority_file.path
socket = OpenSSL::SSL::SSLSocket.new(TCPSocket.new("127.0.0.1", listener.addr[1]), context)
socket.connect
p socket.read
serving.join
p received.uniq
