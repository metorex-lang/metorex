# Two certificates that were never signed have no encoding to compare, and
# count as equal.
require "openssl"

first = OpenSSL::X509::Certificate.new
second = OpenSSL::X509::Certificate.new
p first == second
p [first] == [second]
second.serial = 5
p first == second
