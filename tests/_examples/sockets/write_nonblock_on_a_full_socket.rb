# write_nonblock writes what the socket takes and raises
# IO::EAGAINWaitWritable once it takes nothing, for a TCP socket and a pair
# of UNIX sockets alike, and an empty write answers 0.
require "socket"

server = TCPServer.new("127.0.0.1", 0)
client = TCPSocket.new("127.0.0.1", server.local_address.ip_port)
peer = server.accept
written = 0
begin
  loop { written += client.write_nonblock("X" * 1024) }
rescue IO::WaitWritable => error
  p error.class, error.message, written > 0
end
p client.write_nonblock("", exception: false)

left, right = UNIXSocket.pair
begin
  loop { left.write_nonblock("X" * 1024) }
rescue IO::WaitWritable => error
  p error.class, error.message
end
[client, peer, server, left, right].each(&:close)
