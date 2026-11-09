require "socket"
server = TCPServer.new("127.0.0.1", 0)
client = TCPSocket.new("127.0.0.1", server.addr[1])
accepted = server.accept
p client.write("HTTP/1.1 ", 200, " OK\r\n")
client.close
p accepted.read
accepted.close
server.close
