# A subclass of TCPSocket that sets TCP_NODELAY writes to its peer with
# syswrite, and TCP and UNIX sockets read and write with sysread and syswrite.
require "socket"

class RequestSocket < TCPSocket
  def send_request(text)
    syswrite(text)
    self
  end
end

server = TCPServer.new("127.0.0.1", 0)
client = RequestSocket.new("127.0.0.1", server.addr[1])
client.setsockopt(Socket::IPPROTO_TCP, Socket::TCP_NODELAY, 1)
peer = server.accept
p client.send_request("GET / HTTP/1.1\r\n\r\n").class
p peer.readpartial(100)
p client.getsockopt(Socket::IPPROTO_TCP, Socket::TCP_NODELAY).bool
p peer.syswrite("HTTP/1.1 200 OK\r\n\r\n")
p client.sysread(100)
left, right = UNIXSocket.pair
p left.syswrite("ping")
p right.sysread(10)
