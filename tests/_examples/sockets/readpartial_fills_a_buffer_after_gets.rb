require 'socket'

server = TCPServer.new('127.0.0.1', 0)
client = TCPSocket.new('127.0.0.1', server.addr[1])
peer = server.accept
peer.write("HTTP/1.1 200 OK\r\n\r\nok")
peer.close
client.gets("\r\n")
client.gets("\r\n")
buffer = ''.dup.force_encoding('BINARY')
answer = client.readpartial(2, buffer)
p buffer
p answer.equal?(buffer)
p buffer.encoding
