# With no other thread running, `accept` waits for a client process and
# `eof?` and `readpartial` wait for a server process, however long each
# takes, as RuboCop's server and its client do. A listener a parent hands on
# is taken up with `for_fd`.
require "rbconfig"
require "socket"

server = TCPServer.new("127.0.0.1", 0)
port = server.addr[1]
answering = spawn(RbConfig.ruby, "-rsocket", "-e", <<~RUBY, server.fileno => server.fileno)
  server = TCPServer.for_fd(#{server.fileno})
  connection = server.accept
  connection.read
  sleep 2.5
  connection.write "checked"
  connection.close
RUBY
received = +""
sleep 2.5
TCPSocket.open("127.0.0.1", port) do |socket|
  socket.write "request"
  socket.close_write
  received << socket.readpartial(4096) until socket.eof?
end
p received
Process.wait(answering)
