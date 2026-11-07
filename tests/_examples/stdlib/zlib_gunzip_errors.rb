require "zlib"

member = Zlib.gzip("hello").b
crc = member.dup
crc.setbyte(-6, crc.getbyte(-6) ^ 1)
length = member.dup
length.setbyte(-2, 1)
method = member.dup
method.setbyte(2, 9)
[member, member + "x", "not a member", member[0, 12], member[0, 9], crc, length, method,
 member[0, member.bytesize - 3], "\x1f\x8b".b, "".b, "\x1f".b].each do |text|
  p Zlib.gunzip(text)
rescue Zlib::Error => error
  p [error.class, error.message]
end
