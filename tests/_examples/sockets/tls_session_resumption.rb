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

server_context = OpenSSL::SSL::SSLContext.new
server_context.cert = certificate
server_context.key = key

def exchange(listener, server_context, version, session)
  serving = Thread.new do
    connection = OpenSSL::SSL::SSLSocket.new(listener.accept, server_context)
    connection.accept
    connection.write(connection.read(4).upcase)
    reused = connection.session_reused?
    connection.close
    reused
  end
  sessions = []
  context = OpenSSL::SSL::SSLContext.new
  context.min_version = version
  context.max_version = version
  context.session_cache_mode = OpenSSL::SSL::SSLContext::SESSION_CACHE_CLIENT
  context.session_new_cb = ->(pair) { sessions << pair.last }
  socket = OpenSSL::SSL::SSLSocket.new(TCPSocket.new("127.0.0.1", listener.addr[1]), context)
  socket.session = session if session
  socket.connect
  socket.write("ping")
  reply = socket.read
  p [reply, socket.ssl_version, socket.session_reused?, serving.value, sessions.size, socket.peer_cert.subject.to_s]
  socket.close
  sessions
end

listener = TCPServer.new("127.0.0.1", 0)
[OpenSSL::SSL::TLS1_2_VERSION, OpenSSL::SSL::TLS1_3_VERSION].each do |version|
  sessions = exchange(listener, server_context, version, nil)
  pem = sessions.first.to_pem
  puts pem.lines.first
  copy = OpenSSL::SSL::Session.new(pem)
  p copy == sessions.first
  exchange(listener, server_context, version, copy)
end

begin
  OpenSSL::SSL::Session.new("junk")
rescue ArgumentError => error
  p error.message
end
