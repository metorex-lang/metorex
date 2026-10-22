require "socket"

listener = TCPServer.new("127.0.0.1", 0)
client = Socket.tcp("127.0.0.1", listener.addr[1])
served = listener.accept
writer = Thread.new do
  served.write("* 1 FETCH (BODY[] {12345}\r\n" + "a" * 12_345 + ")\r\nRUBY0001 OK done\r\n")
end
line = client.gets("\r\n", 12_375)
p line
literal = String.new(capacity: 12_345)
read = client.read(12_345, literal)
p read.bytesize, read.equal?(literal), read.count("a")
p client.gets("\r\n", 100), client.gets("\r\n", 100)
writer.join
[client, served, listener].each(&:close)
