# A context held to TLS 1.2 on either side agrees to TLS 1.2 with
# ECDHE-RSA-AES128-GCM-SHA256, carries data both ways, and proves a client
# certificate when the server asks for one.
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

store = OpenSSL::X509::Store.new
store.add_cert(cert)

def connect(server_context, client_context)
  server = TCPServer.new("127.0.0.1", 0)
  client = TCPSocket.open("127.0.0.1", server.local_address.ip_port)
  peer = server.accept
  ssl_peer = OpenSSL::SSL::SSLSocket.new(peer, server_context)
  ssl_client = OpenSSL::SSL::SSLSocket.new(client, client_context)
  handshake = Thread.new do
    ssl_client.connect
  rescue OpenSSL::SSL::SSLError => error
    error
  end
  begin
    ssl_peer.accept
  rescue OpenSSL::SSL::SSLError => error
    p [:server, error.message[/state=error: .*/]]
    handshake.join
    return
  end
  client_result = handshake.value
  return p([:client, client_result.message[/state=error: .*/]]) if client_result.is_a?(Exception)
  ssl_client.write("ping")
  p ssl_peer.readpartial(10)
  ssl_peer.write("pong")
  p ssl_client.readpartial(10)
  p [ssl_peer.ssl_version, ssl_client.ssl_version]
  p ssl_client.cipher[1]
  [ssl_peer, ssl_client]
end

def server_context(cert, key)
  context = OpenSSL::SSL::SSLContext.new
  context.cert = cert
  context.key = key
  context
end

held_server = server_context(cert, key)
held_server.max_version = OpenSSL::SSL::TLS1_2_VERSION
connect(held_server, OpenSSL::SSL::SSLContext.new)

held_client = OpenSSL::SSL::SSLContext.new
held_client.max_version = :TLS1_2
connect(server_context(cert, key), held_client)

no_tls_1_3 = OpenSSL::SSL::SSLContext.new
no_tls_1_3.options |= OpenSSL::SSL::OP_NO_TLSv1_3
connect(server_context(cert, key), no_tls_1_3)

checking_client = OpenSSL::SSL::SSLContext.new
checking_client.max_version = :TLS1_2
checking_client.verify_mode = OpenSSL::SSL::VERIFY_PEER
checking_client.cert_store = store
checking_server = server_context(cert, key)
checking_server.verify_mode = OpenSSL::SSL::VERIFY_PEER | OpenSSL::SSL::VERIFY_FAIL_IF_NO_PEER_CERT
checking_server.cert_store = store
checking_client.cert = cert
checking_client.key = key
ssl_peer, = connect(checking_server, checking_client)
p ssl_peer.peer_cert.subject.to_s

newer_server = server_context(cert, key)
newer_server.min_version = :TLS1_3
connect(newer_server, held_client)
