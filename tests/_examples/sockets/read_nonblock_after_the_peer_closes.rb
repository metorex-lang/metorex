require "socket"

server = TCPServer.new("127.0.0.1", 0)
client = TCPSocket.new("127.0.0.1", server.addr[1])
accepted = server.accept
accepted.write("hi")
accepted.close
sleep 0.2
p client.read_nonblock(5)
begin
  p client.read_nonblock(5)
rescue EOFError => error
  puts "#{error.class}: #{error.message}"
end
p client.read_nonblock(5, exception: false)
client.close
server.close
