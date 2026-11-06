# An IPv6 link-local address names the interface it belongs to after a `%`,
# and Addrinfo keeps the name.
require "socket"
held = Addrinfo.ip("fe80::1%lo0")
p held
p held.ip_address
p held.ipv6_linklocal?
