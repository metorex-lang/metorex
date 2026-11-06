require "socket"
p Socket::SOMAXCONN
[[0], ["127.0.0.1", 0], ["localhost", 0]].each do |arguments|
  sockets = Socket.tcp_server_sockets(*arguments)
  p sockets.map { |held| [held.local_address.ip_address, held.local_address.ipv6?] }
  p sockets.map { |held| held.local_address.ip_port }.uniq.size
  p sockets.map { |held| held.getsockopt(:SOCKET, :REUSEADDR).int.positive? }
  sockets.each(&:close)
end
