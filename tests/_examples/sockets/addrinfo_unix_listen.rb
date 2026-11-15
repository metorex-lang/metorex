require "socket"
require "tmpdir"

Dir.mktmpdir do |dir|
  path = File.join(dir, "listen.sock")
  server = Addrinfo.unix(path).listen
  p server.class
  p server.local_address.unix_path == path
  client = UNIXSocket.new(path)
  accepted, = server.accept
  client.write("hello")
  p accepted.read(5)
  [client, accepted, server].each(&:close)
end
