# A socket's to_i is the number fileno answers, for a listener, a client,
# an accepted connection, and a Socket.
require "socket"

server = TCPServer.new("127.0.0.1", 0)
client = TCPSocket.new("127.0.0.1", server.addr[1])
peer = server.accept
raw = Socket.new(:INET, :STREAM)
[server, client, peer, raw].each do |socket|
  p [socket.class, socket.to_i == socket.fileno, socket.to_i.positive?]
end
p({ server.to_i => server.to_i }.size)
