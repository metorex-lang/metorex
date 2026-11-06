require "socket"
require "timeout"

listener = TCPServer.new("127.0.0.1", 0)
go_ahead = Queue.new
server = Thread.new do
  peer = listener.accept
  go_ahead.pop
  peer.write("late\n")
  peer.close
end
client = TCPSocket.new("127.0.0.1", listener.addr[1])
%i[gets readpartial].each do |how|
  begin
    Timeout.timeout(0.2) { how == :gets ? client.gets : client.readpartial(5) }
  rescue Timeout::Error => error
    p [how, error.message]
  end
end
go_ahead << :write
p client.gets
server.join
