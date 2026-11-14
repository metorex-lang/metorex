require "openssl"

# RFC 5869's first test case, then SHA-512 with an empty salt and info.
ikm = ["0b" * 22].pack("H*")
salt = ["000102030405060708090a0b0c"].pack("H*")
info = ["f0f1f2f3f4f5f6f7f8f9"].pack("H*")
p OpenSSL::KDF.hkdf(ikm, salt: salt, info: info, length: 42, hash: "SHA256").unpack1("H*")
p OpenSSL::KDF.hkdf("key", salt: "", info: "", length: 70, hash: OpenSSL::Digest::SHA512.new).unpack1("H*")
[0, 32 * 255 + 1].each do |length|
  OpenSSL::KDF.hkdf("key", salt: "s", info: "i", length: length, hash: "SHA256")
rescue OpenSSL::KDF::KDFError => error
  p error.message
end
p OpenSSL::KDF.hkdf("key", salt: "s", info: "i", length: 32 * 255, hash: "SHA256").bytesize
begin
  OpenSSL::KDF.hkdf("key", salt: "s", info: "i", length: -1, hash: "SHA256")
rescue => error
  p error.class, error.message
end
