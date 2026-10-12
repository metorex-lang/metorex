require "socket"

# A control message names its family, level and type by number or by name,
# with a type's name read within its level.
message = Socket::AncillaryData.new(:INET, :IP, :TTL, "x")
p [message.family == Socket::AF_INET, message.level == Socket::IPPROTO_IP]
p message.type == Socket::IP_TTL
p message.cmsg_is?(:IP, :TTL)
p message

begin
  Socket::AncillaryData.new(:INET, :SOCKET, :MOO, "")
rescue SocketError => problem
  p problem.message
end
begin
  Socket::AncillaryData.new(:UNIX, :IP, :TTL, "")
rescue SocketError => problem
  p problem.message
end

# An integer is carried as a C int.
counted = Socket::AncillaryData.int(:INET, :IP, :TTL, 64)
p counted.int
p counted

# Descriptors handed on over a UNIX socket.
rights = Socket::AncillaryData.unix_rights(STDOUT, STDERR)
p rights.unix_rights == [STDOUT, STDERR]
p rights.data.unpack("i*") == [STDOUT.fileno, STDERR.fileno]
begin
  message.unix_rights
rescue TypeError => problem
  p problem.message
end

# Where a packet was sent and which interface took it.
info = Socket::AncillaryData.ip_pktinfo(Addrinfo.tcp("127.0.0.1", 80), 4, Addrinfo.ip("127.0.0.5"))
address, index, destination = info.ip_pktinfo
p [address.ip_address, address.ip_port, index, destination.ip_address]
info = Socket::AncillaryData.ipv6_pktinfo(Addrinfo.ip("::1"), 3)
p [info.ipv6_pktinfo_addr.ip_address, info.ipv6_pktinfo_ifindex]
