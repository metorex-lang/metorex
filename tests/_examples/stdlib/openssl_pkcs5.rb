# OpenSSL::PKCS5 derives a key with PBKDF2 as OpenSSL::KDF does, under the
# name Active Support's key generator calls.
require "openssl"

derived = OpenSSL::PKCS5.pbkdf2_hmac("maintenance window", "ledger salt", 1000, 32, OpenSSL::Digest::SHA256.new)
p derived.bytesize
p derived.unpack1("H*")
p derived == OpenSSL::KDF.pbkdf2_hmac("maintenance window", salt: "ledger salt", iterations: 1000, length: 32, hash: "sha256")
p OpenSSL::PKCS5.pbkdf2_hmac_sha1("password", "salt", 1, 20).unpack1("H*")
