# A connection with a time limit to a port nothing listens on is refused,
# rather than reported as made.
require "socket"

closed = TCPServer.new("127.0.0.1", 0)
port = closed.addr[1]
closed.close

begin
  Socket.tcp("127.0.0.1", port, connect_timeout: 5)
  puts "connected"
rescue SystemCallError => error
  puts error.class
end

begin
  Addrinfo.tcp("127.0.0.1", port).connect(timeout: 5)
  puts "connected"
rescue SystemCallError => error
  puts error.class
end
