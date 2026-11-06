# What the non-blocking reads answer once the other end has closed.
require "socket"
server = TCPServer.new("127.0.0.1", 0)
client = Socket.tcp("127.0.0.1", server.addr[1])
peer = server.accept
peer.write("ab")
peer.close
buffer = +"old"
client.wait_readable
p client.read_nonblock(10, buffer)
p buffer
client.wait_readable
p((client.read_nonblock(10, buffer) rescue $!.class))
p buffer
p client.read_nonblock(10, buffer, exception: false)
p client.recv_nonblock(10)
p client.recv(10)
