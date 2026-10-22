require "socket"

listener = TCPServer.new("127.0.0.1", 0)
client = TCPSocket.new("127.0.0.1", listener.addr[1])
served = listener.accept
p listener.sync, client.sync, served.sync
p UNIXSocket.pair.map(&:sync)
[client, served, listener].each(&:close)
