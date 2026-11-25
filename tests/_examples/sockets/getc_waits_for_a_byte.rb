require "socket"

server = TCPServer.new("127.0.0.1", 0)
client = TCPSocket.new("127.0.0.1", server.addr[1])
accepted = server.accept

read = Queue.new
reader = Thread.new do
  3.times { read << client.getc }
end

sleep 0.2
p read.empty?
accepted.write("ab")
p [read.pop, read.pop]
accepted.write("c")
p read.pop
reader.join

accepted.write("d\xffe".b)
p client.getbyte
p client.readchar
p client.readbyte
accepted.close
p client.getc
p client.getbyte
begin
  client.readchar
rescue EOFError => error
  puts "#{error.class}: #{error.message}"
end
begin
  client.readbyte
rescue EOFError => error
  puts "#{error.class}: #{error.message}"
end

server.close
client.close

near, far = UNIXSocket.pair
far.write("xy")
p [near.getc, near.getbyte]
far.close
p near.getc
near.close
