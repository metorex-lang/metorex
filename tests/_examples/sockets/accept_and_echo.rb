# A server thread accepts a connection, reads what the client wrote, and
# answers it, while the main thread writes, half-closes, and reads the answer.
require "socket"

server = TCPServer.new("127.0.0.1", 0)
port = server.addr[1]
data = nil
thread = Thread.new do
  client = server.accept
  data = client.read(5)
  client << "goodbye"
  client.close
end
Thread.pass while thread.status && thread.status != "sleep"

socket = TCPSocket.new("127.0.0.1", port)
socket.write("hello")
socket.shutdown(1)
p socket.read
thread.join
p data
socket.close
server.close
