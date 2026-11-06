# A non-blocking connect whose completion is read from SO_ERROR, with no
# second connect call, leaves a socket that writes and reads.
require "socket"
server = TCPServer.new("127.0.0.1", 0)
address = Socket.sockaddr_in(server.addr[1], "127.0.0.1")
client = Socket.new(:INET, :STREAM)
begin
  client.connect_nonblock(address)
  p :connected_at_once
rescue IO::WaitWritable
  p :waiting
  client.wait_writable
end
p client.getsockopt(:SOCKET, :ERROR).int
peer = server.accept
p client.write("ping")
p peer.read(4)
peer.write("pong")
p client.read(4)
