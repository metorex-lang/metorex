# A socket's gets takes a separator and a byte limit, as net-imap reads a
# response line no longer than its limit.
require "socket"

server = TCPServer.new("127.0.0.1", 0)
client = TCPSocket.new("127.0.0.1", server.addr[1])
accepted = server.accept
client.write "* OK ready\r\n* 2 EXISTS\r\nA001 OK done\r\n"
client.close
p accepted.gets("\r\n", 100)
p accepted.gets("\r\n", 4)
p accepted.gets("\r\n", 100)
p accepted.gets("\r\n")
p accepted.gets("\r\n")

# A Socket made with Socket.tcp reads the same way.
server = TCPServer.new("127.0.0.1", 0)
client = Socket.tcp("127.0.0.1", server.addr[1])
accepted = server.accept
accepted.write "* OK ready\r\n"
accepted.close
p client.gets("\r\n", 4)
p client.gets("\r\n", 100)
