require "socket"

server = TCPServer.new("::1", 0)
port = server.addr[1]
Thread.new { server.accept.print("* OK ready\r\n") }

client = Socket.tcp("localhost", port, connect_timeout: 5)
p client.gets
p client.remote_address.ipv6?
client.close

begin
  Socket.tcp("127.0.0.1", port)
rescue Errno::ECONNREFUSED => error
  p error.class
end
