require "openssl"
require "socket"

fixtures = File.join(__dir__, "tls_fixtures")
context = OpenSSL::SSL::SSLContext.new
context.ca_file = File.join(fixtures, "cacert.pem")
context.key = File.open(File.join(fixtures, "server.key")) { |file| OpenSSL::PKey::RSA.new(file) }
context.cert = File.open(File.join(fixtures, "server.crt")) { |file| OpenSSL::X509::Certificate.new(file) }
listener = TCPServer.new("127.0.0.1", 0)
secure = OpenSSL::SSL::SSLServer.new(listener, context)
server = Thread.start do
  socket = secure.accept
  socket.print("* OK test server\r\n")
  line = socket.gets
  socket.print("* BYE terminating connection\r\n")
  socket.close
  line
end

client_context = OpenSSL::SSL::SSLContext.new
client_context.set_params(verify_mode: OpenSSL::SSL::VERIFY_NONE)
client = OpenSSL::SSL::SSLSocket.new(Socket.tcp("127.0.0.1", listener.addr[1], connect_timeout: 5), client_context)
client.sync_close = true
client.hostname = "127.0.0.1"
# Net::Protocol connects this way when it has an open timeout.
loop do
  case client.connect_nonblock(exception: false)
  when :wait_readable then client.to_io.wait_readable(5)
  when :wait_writable then client.to_io.wait_writable(5)
  else break
  end
end
p client.gets("\r\n", 536_870_912)
client.print("RUBY0001 LOGOUT\r\n")
p client.gets("\r\n", 536_870_912)
p client.gets("\r\n", 536_870_912)
client.close
p server.value
secure.close
