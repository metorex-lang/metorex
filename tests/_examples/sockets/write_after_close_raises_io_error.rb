require "socket"

server = TCPServer.new("127.0.0.1", 0)
client = TCPSocket.new("127.0.0.1", server.addr[1])
client.close
begin
  client.write("hello")
rescue IOError => error
  puts "#{error.class}: #{error.message}"
end

left, right = UNIXSocket.pair
left.close
begin
  left.write("hello")
rescue IOError => error
  puts "#{error.class}: #{error.message}"
end
server.close
right.close
