# TCPServer.new and TCPServer.open with nil for the port listen on a free
# port of the host named, as with 0.
require "socket"

server = TCPServer.new("127.0.0.1", nil)
p [server.addr[1].positive?, server.addr[3]]
server.close
p TCPServer.open("127.0.0.1", nil) { |held| held.addr[1].positive? }
alone = TCPServer.new(0)
p alone.addr[1].positive?
alone.close
