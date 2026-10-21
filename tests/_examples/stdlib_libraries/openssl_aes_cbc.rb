# OpenSSL::Cipher runs AES in CBC mode: FIPS-197's example vectors as one
# block from a zero IV, then text padded the way PKCS#7 pads it, fed in
# pieces, and read back. OpenSSL::SSL names the verify modes OpenSSL has.
require "openssl"

def hex(text) = text.unpack1("H*")
def bytes(hex) = [hex].pack("H*")

{ "AES-128-CBC" => "000102030405060708090a0b0c0d0e0f",
  "AES-256-CBC" => "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f" }.each do |name, key|
  cipher = OpenSSL::Cipher.new(name).encrypt
  cipher.key = bytes(key)
  cipher.iv = "\0" * 16
  cipher.padding = 0
  p hex(cipher.update(bytes("00112233445566778899aabbccddeeff")) + cipher.final)
end

key = "k" * 32
iv = "i" * 16
cipher = OpenSSL::Cipher.new("aes-256-cbc").encrypt
cipher.key = key
cipher.iv = iv
sealed = cipher.update("invoice 1042 is due ") + cipher.update("on 2026-11-01") + cipher.final
p hex(sealed)

reader = OpenSSL::Cipher.new("AES-256-CBC").decrypt
reader.key = key
reader.iv = iv
p reader.update(sealed[0, 7]) + reader.update(sealed[7..]) + reader.final

reader = OpenSSL::Cipher.new("AES-256-CBC").decrypt
reader.key = "x" * 32
reader.iv = iv
begin
  reader.update(sealed) + reader.final
rescue OpenSSL::Cipher::CipherError => error
  p error.class
end
p OpenSSL::Cipher.new("AES-192-CBC").key_len
p OpenSSL::SSL.constants.grep(/VERIFY/).sort.map { |name| [name, OpenSSL::SSL.const_get(name)] }
