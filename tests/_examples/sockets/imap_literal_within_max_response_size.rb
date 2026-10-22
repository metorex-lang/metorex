require "net/imap"

listener = TCPServer.new("localhost", 0)
server = Thread.new do
  socket = listener.accept
  socket.print "* PREAUTH ready\r\n"
  while (line = socket.gets("\r\n"))
    tag, command = line.split
    case command
    when "NOOP"
      socket.print "* 1 FETCH (BODY[] {1234}\r\n" + "a" * 1_234 + ")\r\n"
      socket.print "#{tag} OK done\r\n"
    when "LOGOUT"
      socket.print "* BYE bye\r\n#{tag} OK done\r\n"
      break
    end
  end
  socket.close
end

imap = Net::IMAP.new("localhost", port: listener.addr[1])
imap.max_response_size = 1_234 + 30
begin
  imap.noop
  p imap.responses("FETCH").first.message.size
rescue => error
  p error.class, error.message
end
imap.logout rescue p $!.class
imap.disconnect
server.join
listener.close
