# A connection made while another thread waits in `accept`, which runs that
# thread while the connection is made. The client then writes and closes its
# half through its own end, after the server's end has closed.
require "socket"

server = TCPServer.new("127.0.0.1", 0)
port = server.addr[1]
received = nil
thread = Thread.new do
  client = server.accept
  received = client.read(5)
  client.close
end
Thread.pass while thread.status && thread.status != "sleep"
socket = TCPSocket.new("127.0.0.1", port)
socket.write("hello")
socket.shutdown(1)
thread.join
p [received, socket.closed?]
socket.close
server.close
