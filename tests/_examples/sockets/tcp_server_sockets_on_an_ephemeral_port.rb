# Socket.tcp_server_sockets with port 0 opens a listener for each address
# the host name has, all on the one port the system picked.
require "socket"
sockets = Socket.tcp_server_sockets("localhost", 0)
p sockets.empty?
ports = sockets.map { |socket| socket.local_address.ip_port }
p ports.uniq.size
p ports.first.positive?
p sockets.first.class
sockets.each(&:close)

# A listener handed on by its descriptor, as WEBrick does, keeps working
# after the Socket it came from is closed without closing the descriptor.
listeners = Socket.tcp_server_sockets("localhost", 0).map do |socket|
  socket.autoclose = false
  server = TCPServer.for_fd(socket.fileno)
  socket.close
  server
end
p listeners.size.positive?
p listeners[0].class
p listeners[0].addr[1].positive?
listeners.each(&:close)
