require "net/imap"

fixtures = File.join(__dir__, "tls_fixtures")
context = OpenSSL::SSL::SSLContext.new
context.key = File.open(File.join(fixtures, "server.key")) { |file| OpenSSL::PKey::RSA.new(file) }
context.cert = File.open(File.join(fixtures, "server.crt")) { |file| OpenSSL::X509::Certificate.new(file) }
listener = TCPServer.new("127.0.0.1", 0)
secure = OpenSSL::SSL::SSLServer.new(listener, context)
server = Thread.start do
  socket = secure.accept
  socket.print("* OK test server\r\n")
  line = socket.gets
  socket.print("* BYE terminating connection\r\n")
  socket.print("RUBY0001 OK LOGOUT completed\r\n")
  socket.close
  line
end

imap = Net::IMAP.new("127.0.0.1", port: listener.addr[1], ssl: { verify_mode: OpenSSL::SSL::VERIFY_NONE })
p imap.greeting.data.text, imap.tls_verified?
imap.logout
imap.disconnect
p server.value
secure.close
