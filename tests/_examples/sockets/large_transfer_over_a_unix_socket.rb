# Five megabytes written through a Unix socket arrive whole, read the way
# WEBrick reads a request body.
require "socket"
require "tmpdir"
Dir.mktmpdir do |directory|
  path = File.join(directory, "server.sock")
  server = UNIXServer.new(path)
  reader = Thread.start do
    peer = server.accept
    received = +""
    while received.bytesize < 5_000_000
      chunk = peer.readpartial([65536, 5_000_000 - received.bytesize].min)
      received << chunk
    end
    peer.close
    received.bytesize
  end
  client = UNIXSocket.new(path)
  client.write("x" * 5_000_000)
  p reader.value
  client.close
  server.close
end
