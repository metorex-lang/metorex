require "openssl"
key = (1..32).map(&:chr).join.b
iv = "\x10".b * 12
c = OpenSSL::Cipher.new("aes-256-gcm")
p [c.iv_len, c.key_len, c.block_size, c.authenticated?, c.name]
c.encrypt
c.key = key
c.iv = iv
c.auth_data = "header"
out = c.update("The quick brown fox jumps over the lazy dog") + c.final
p out.unpack1("H*"), c.auth_tag.unpack1("H*"), c.auth_tag(12).unpack1("H*")
d = OpenSSL::Cipher.new("aes-256-gcm").decrypt
d.key = key
d.iv = iv
d.auth_tag = c.auth_tag
d.auth_data = "header"
p d.update(out) + d.final
e = OpenSSL::Cipher.new("aes-128-gcm").encrypt
e.key = key[0, 16]
e.iv_len = 16
e.iv = "\x01".b * 16
p (e.update("x" * 40) + e.final).unpack1("H*"), e.auth_tag.unpack1("H*")
f = OpenSSL::Cipher.new("aes-256-gcm").decrypt
f.key = key
f.iv = iv
f.auth_tag = "\0" * 16
f.update(out)
begin
  f.final
rescue OpenSSL::Cipher::CipherError => error
  p error.message
end
g = OpenSSL::Cipher.new("aes-256-gcm").encrypt
g.key = key
g.iv = iv
p g.final.bytesize, g.auth_tag.unpack1("H*")
p OpenSSL::Cipher.new("aes-256-cbc").authenticated?
begin
  OpenSSL::Cipher.new("aes-256-cbc").auth_tag
rescue => error
  p error.class, error.message
end
