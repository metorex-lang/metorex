require "openssl"
require "socket"

server = TCPServer.new("127.0.0.1", 0)
[:SSL3, :TLS1, :TLS1_1].each do |version|
  client = TCPSocket.new("127.0.0.1", server.addr[1])
  accepted = server.accept
  context = OpenSSL::SSL::SSLContext.new
  context.max_version = version
  begin
    OpenSSL::SSL::SSLSocket.new(client, context).connect
  rescue OpenSSL::SSL::SSLError => error
    p [version, error.message.sub(/peeraddr=\S+ /, ""), accepted.read(7)]
  end
end
