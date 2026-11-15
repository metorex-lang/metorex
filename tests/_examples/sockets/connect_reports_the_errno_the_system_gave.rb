require "socket"

server = TCPServer.new("127.0.0.1", 0)
port = server.addr[1]
server.close
begin
  TCPSocket.new("127.0.0.1", port)
rescue SystemCallError => error
  p error.class
end

# macOS refuses port 0 as an address it cannot assign, where Linux refuses
# the connection.
begin
  TCPSocket.new("127.0.0.1", 0)
rescue SystemCallError => error
  p RUBY_PLATFORM.include?("darwin") ? error.class : Errno::EADDRNOTAVAIL
end
