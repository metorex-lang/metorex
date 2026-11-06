# Socket reads lines through IO#readline, and gets takes chomp.
require "socket"
server = TCPServer.new("127.0.0.1", 0)
client = Socket.tcp("127.0.0.1", server.addr[1])
peer = server.accept
peer.write("first\nsecond\nthird\n")
peer.close
p client.readline
p client.gets(chomp: true)
p client.readline(chomp: true)
p client.gets
