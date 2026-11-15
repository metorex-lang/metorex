# A TCPServer taken up from the descriptor of a unix listener, as a service
# manager hands one over, answers the path it listens on.
require "socket"
require "tmpdir"
Dir.mktmpdir do |dir|
  path = File.join(dir, "a.sock")
  sock = Addrinfo.unix(path).listen
  held = TCPServer.for_fd(sock.fileno)
  p held.class
  p held.getsockname.bytesize > 2
  p Socket.unpack_sockaddr_un(held.getsockname) == path
  p held.local_address.unix?
end
