require "socket"

# `open_timeout:` limits how long a connection may take to open.
server = TCPServer.new "127.0.0.1", 0
port = server.addr[1]
client = TCPSocket.new "127.0.0.1", port, open_timeout: 5
p client.class
expected = "#<TCPSocket:fd #{client.fileno}, AF_INET, 127.0.0.1, #{client.addr[1]}>"
p client.inspect == expected
client.close
p client.inspect

reached = Socket.tcp "127.0.0.1", port, open_timeout: 5
p reached.class
reached.close

# It cannot be given beside a limit on one part of the connection.
begin
  TCPSocket.new "127.0.0.1", port, open_timeout: 1, connect_timeout: 1
rescue ArgumentError => problem
  p problem.message
end

# A closed UDP socket shows only that it is closed.
datagrams = UDPSocket.new
datagrams.bind "127.0.0.1", 0
datagrams.close
p datagrams.inspect
server.close

# A port nothing listens on refuses the connection, and a limit that has
# already passed times it out.
closed = TCPServer.new "127.0.0.1", 0
closed_port = closed.addr[1]
closed.close
begin
  TCPSocket.new "127.0.0.1", closed_port, open_timeout: 5
rescue Errno::ECONNREFUSED => problem
  p problem.class
end
begin
  TCPSocket.new "192.0.2.1", 80, open_timeout: 0
rescue IO::TimeoutError => problem
  p problem.message
end
