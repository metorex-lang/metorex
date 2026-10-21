# TCPServer#accept waits for a client to connect, however long that takes,
# with no other thread running, as a server process does.
require "rbconfig"
require "socket"

server = TCPServer.new("127.0.0.1", 0)
port = server.addr[1]
client = spawn(RbConfig.ruby, "-rsocket", "-e",
               "sleep 0.3; TCPSocket.open('127.0.0.1', #{port}) { |socket| socket.write 'ping' }")
accepted = server.accept
p accepted.read
Process.wait(client)
