# Serves repository/ over HTTP on a port of its own, printing the port, so
# Bundler installs from it the way it does from rubygems.org. A path with no
# file of that name answers 404.
require "socket"

repository = File.join(__dir__, "repository")
server = TCPServer.new("127.0.0.1", 0)
puts server.addr[1]
$stdout.flush
loop do
  next unless IO.select([server], nil, nil, 60)
  client = server.accept
  request = client.gets.to_s
  nil until client.gets.to_s.strip.empty?
  path = request.split[1].to_s.split("?").first.to_s
  file = File.expand_path(path.delete_prefix("/"), repository)
  if file.start_with?(repository + "/") && File.file?(file)
    body = File.binread(file)
    client.write "HTTP/1.1 200 OK\r\nContent-Length: #{body.bytesize}\r\nConnection: close\r\n\r\n"
    client.write body
  else
    client.write "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
  end
  client.close
end
