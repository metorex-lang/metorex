require "ipaddr"
require "resolv"
p IPAddr.new("127.0.0.1").loopback?
p IPAddr.new("::1").loopback?
p IPAddr.new("169.254.170.2") == IPAddr.new("169.254.170.2")
p IPAddr.new("fd00:ec2::23") == IPAddr.new("fd00:ec2::23")
p IPAddr.new("169.254.170.23")
p Resolv.getaddresses("localhost").sort
p IPAddr.new("[::1]")
