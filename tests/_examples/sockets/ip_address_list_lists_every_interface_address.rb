# Socket.ip_address_list names the address of every interface that has one
# in an internet family, the loopback ones among them, as Addrinfo.
require "socket"
listed = Socket.ip_address_list
p listed.all?(Addrinfo)
p listed.map(&:ip_address).include?("127.0.0.1")
from_interfaces = Socket.getifaddrs.filter_map { |interface| interface.addr&.ip? ? interface.addr.ip_address : nil }
p listed.map(&:ip_address) == from_interfaces
p listed.select(&:ipv6_linklocal?).all? { |held| held.ip_address.include?("%") }
