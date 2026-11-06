# A server's servername_cb hears the name the client asked for, and the
# context it answers with is the one whose certificate the client receives.
require "openssl"
require "socket"

fixtures = File.join(__dir__, "tls_fixtures")
key = File.open(File.join(fixtures, "server.key")) { |file| OpenSSL::PKey::RSA.new(file) }
default_context = OpenSSL::SSL::SSLContext.new
default_context.key = key
default_context.cert = File.open(File.join(fixtures, "server.crt")) { |file| OpenSSL::X509::Certificate.new(file) }

named = OpenSSL::X509::Certificate.new
named.version = 2
named.serial = 2
named.subject = named.issuer = OpenSSL::X509::Name.parse("/CN=excon")
named.public_key = key.public_key
named.not_before = Time.now - 60
named.not_after = Time.now + 3600
named.sign(key, OpenSSL::Digest.new("SHA256"))
named_context = OpenSSL::SSL::SSLContext.new
named_context.key = key
named_context.cert = named

heard = []
default_context.servername_cb = lambda do |(socket, name)|
  heard << [socket.class, name]
  name == "excon" ? named_context : nil
end

listener = TCPServer.new("127.0.0.1", 0)
server = Thread.start do
  2.times.map do
    secure = OpenSSL::SSL::SSLSocket.new(listener.accept, default_context)
    secure.accept
    secure.write("ok\n")
    host = secure.hostname
    secure.close
    host
  end
end

%w[excon other].each do |name|
  client_context = OpenSSL::SSL::SSLContext.new
  client_context.set_params(verify_mode: OpenSSL::SSL::VERIFY_NONE)
  client = OpenSSL::SSL::SSLSocket.new(TCPSocket.new("127.0.0.1", listener.addr[1]), client_context)
  client.hostname = name
  client.connect
  p [name, client.peer_cert.subject.to_s, client.gets]
  client.close
end
p server.value
p heard
listener.close
