# A TLS client that reads with read_nonblock and waits in IO.select between
# reads, against a server that wraps each accepted socket itself and leaves
# the connection open after it answers.
require "openssl"
require "socket"

fixtures = File.join(__dir__, "tls_fixtures")
context = OpenSSL::SSL::SSLContext.new
context.key = File.open(File.join(fixtures, "server.key")) { |file| OpenSSL::PKey::RSA.new(file) }
context.cert = File.open(File.join(fixtures, "server.crt")) { |file| OpenSSL::X509::Certificate.new(file) }
listener = TCPServer.new("127.0.0.1", 0)
server = Thread.start do
  plain = listener.accept
  secure = OpenSSL::SSL::SSLSocket.new(plain, context)
  secure.sync_close = true
  secure.accept
  request = secure.gets
  secure.write("HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok")
  IO.select([secure], nil, nil, 5)
  secure.close
  request
end

client_context = OpenSSL::SSL::SSLContext.new
client_context.set_params(verify_mode: OpenSSL::SSL::VERIFY_NONE)
client = OpenSSL::SSL::SSLSocket.new(Socket.tcp("127.0.0.1", listener.addr[1]), client_context)
client.sync_close = true
loop do
  case client.connect_nonblock(exception: false)
  when :wait_readable then IO.select([client], nil, nil, 5)
  when :wait_writable then IO.select(nil, [client], nil, 5)
  else break
  end
end
client.write_nonblock("GET / HTTP/1.1\r\n\r\n")
buffer = +""
until buffer.end_with?("ok")
  begin
    buffer << client.read_nonblock(1024)
  rescue IO::WaitReadable
    p :timed_out unless IO.select([client], nil, nil, 5)
  end
end
p buffer
client.close
p server.value
listener.close
