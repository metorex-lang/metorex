# A Socket in the Unix family connects to a path through connect and
# connect_nonblock, and a second connect is refused.
require "socket"
require "tmpdir"
Dir.mktmpdir do |directory|
  path = File.join(directory, "server.sock")
  server = UNIXServer.new(path)
  [:connect, :connect_nonblock].each do |how|
    client = Socket.new(:UNIX, :STREAM)
    p client.public_send(how, Socket.sockaddr_un(path))
    peer = server.accept
    client.write("hello")
    p peer.read(5)
    p((client.connect(Socket.sockaddr_un(path)) rescue $!.class))
    client.close
    peer.close
  end
  server.close
end
