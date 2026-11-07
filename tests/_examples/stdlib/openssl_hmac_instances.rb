require "openssl"

hmac = OpenSSL::HMAC.new("ledger key", "SHA256")
hmac << "line one"
hmac.update(" and two")
p hmac.hexdigest == OpenSSL::HMAC.hexdigest("SHA256", "ledger key", "line one and two")
p hmac.digest.bytesize, hmac.to_s == hmac.hexdigest, hmac.inspect == hmac.hexdigest, hmac.base64digest.size
copy = hmac.dup
copy << " and three"
p hmac.hexdigest == OpenSSL::HMAC.hexdigest("SHA256", "ledger key", "line one and two")
p hmac == OpenSSL::HMAC.new("ledger key", "SHA256").update("line one and two"), hmac == copy
hmac.reset
p hmac.hexdigest == OpenSSL::HMAC.hexdigest("SHA256", "ledger key", "")
p OpenSSL::HMAC.new("key", OpenSSL::Digest.new("SHA1")).update("abc").hexdigest
begin
  OpenSSL::HMAC.new("key")
rescue ArgumentError => error
  p error.class
end
