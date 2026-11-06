require "ipaddr"
network = IPAddr.new("192.168.0.1/32")
p network.include?("192.168.0.1")
p network.include?("::1")
begin
  network.include?("example.com")
rescue IPAddr::InvalidAddressError => error
  p error.class
end
p IPAddr.new("192.168.0.0/24").include?(IPAddr.new("192.168.0.0/16"))
p IPAddr.new("192.168.0.0/16").include?(IPAddr.new("192.168.4.0/24"))
