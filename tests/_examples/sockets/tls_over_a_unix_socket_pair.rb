require "openssl"
require "socket"

fixtures = File.join(__dir__, "tls_fixtures")
server_end, client_end = UNIXSocket.pair
p client_end.read_nonblock(10, exception: false)

server = Thread.new do
  context = OpenSSL::SSL::SSLContext.new
  context.add_certificate(
    OpenSSL::X509::Certificate.new(File.read(File.join(fixtures, "server.crt"))),
    OpenSSL::PKey.read(File.read(File.join(fixtures, "server.key"))),
    [OpenSSL::X509::Certificate.new(File.read(File.join(fixtures, "cacert.pem")))]
  )
  socket = OpenSSL::SSL::SSLSocket.new(server_end, context)
  socket.sync_close = true
  socket.accept
  socket.puts "220 ready"
  line = socket.gets
  socket.close
  line
rescue OpenSSL::SSL::SSLError => error
  error.message
end

store = OpenSSL::X509::Store.new
store.add_file(File.join(fixtures, "cacert.pem"))
context = OpenSSL::SSL::SSLContext.new
context.set_params(cert_store: store)
socket = OpenSSL::SSL::SSLSocket.new(client_end, context)
socket.hostname = "localhost"
socket.sync_close = true
loop do
  case socket.connect_nonblock(exception: false)
  when :wait_readable then socket.to_io.wait_readable(5)
  when :wait_writable then socket.to_io.wait_writable(5)
  else break
  end
end
p socket.verify_result, socket.peer_cert_chain.map { |held| held.subject.to_s }
p socket.gets
socket.puts "QUIT"
p server.value
socket.close
