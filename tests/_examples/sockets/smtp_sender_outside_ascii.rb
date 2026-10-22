require "net/smtp"

servers = Socket.tcp_server_sockets("localhost", 0)
port = servers[0].local_address.ip_port
commands = []
server = Thread.start do
  socket = Socket.accept_loop(servers) { |accepted, _| break accepted }
  socket.puts "220 ready\r\n"
  while (line = socket.gets)
    commands << line.encode("utf-8", "utf-8")
    case line.chomp
    when /\AEHLO / then socket.puts "220-servername\r\n220 SMTPUTF8\r\n"
    when "QUIT"
      socket.puts "221 2.0.0 Bye\r\n"
      break
    when "DATA" then socket.puts "354 End data\r\n"
    when "." then socket.puts "250 2.0.0 Ok\r\n"
    else socket.puts "250 2.1.0 Ok\r\n" unless commands.include?("DATA\r\n")
    end
  end
  socket.close
end

smtp = Net::SMTP.start("localhost", port)
smtp.send_message("message", "rené@example.com", "foo@example.com")
smtp.finish
server.join
servers.each(&:close)
p commands
