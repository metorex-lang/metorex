require "openssl"

key = "clé secrète ünïcode"
p OpenSSL::HMAC.hexdigest("SHA256", key, "Grace O’Malley")
p OpenSSL::HMAC.hexdigest("SHA1", "k" * 100, "données")
p OpenSSL::HMAC.digest("SHA256", "\xFF\x00".b, "plain").bytesize
