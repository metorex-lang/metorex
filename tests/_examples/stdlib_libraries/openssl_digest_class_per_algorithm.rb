require "openssl"

p OpenSSL::Digest::MD5.hexdigest("a")
p OpenSSL::Digest::SHA256.digest("a").bytesize
p OpenSSL::Digest::SHA1.base64digest("a")
p OpenSSL::Digest.hexdigest("MD5", "a")
