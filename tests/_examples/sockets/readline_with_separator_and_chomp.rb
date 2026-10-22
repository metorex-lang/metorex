require "socket"

TCPServer.open("127.0.0.1", 0) do |server|
  TCPSocket.open("127.0.0.1", server.addr[1]) do |socket|
    session = server.accept
    session << "<foo><bar>\nline two\n"
    p socket.read(3)
    p socket.readline(">")
    p socket.gets(">", chomp: true)
    p socket.readline(chomp: true)
    p socket.gets("o", 3)
    session.close
  end
end
