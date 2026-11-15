# A socket made with Socket.new and bound, a UDP socket, and an IO#dup copy
# are closed in a program run with exec, so a child does not hold a port
# the parent bound.
require "socket"

stream = Socket.new(:INET, :STREAM)
stream.bind(Addrinfo.tcp("127.0.0.1", 0))
datagram = UDPSocket.new
reader, writer = IO.pipe
copy = reader.dup
held = { stream: stream, datagram: datagram, copy: copy }
held.each { |name, io| p [name, io.close_on_exec?] }
held.each do |name, io|
  check = "begin; IO.for_fd(#{io.fileno}, autoclose: false); print :open; rescue SystemCallError; print :closed; end"
  p [name, IO.popen([RbConfig.ruby, "-e", check], &:read)]
end
port = stream.connect_address.ip_port
stream.close
again = TCPServer.new("127.0.0.1", port)
p again.addr[1] == port
