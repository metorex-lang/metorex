require "openssl"
number = OpenSSL::BN.new(255)
p number.to_bn.equal?(number), 255.to_bn == number, 255.to_bn.class
