two_bytes = "\xE9\x5F".dup.force_encoding("GB18030")
p two_bytes.encode("UTF-8"), two_bytes.length, two_bytes.valid_encoding?

four_bytes = "\x81\x30\x81\x30\x84\x31\xA4\x39\x90\x30\x81\x30\xE3\x32\x9A\x35".dup.force_encoding("GB18030")
p four_bytes.encode("UTF-8").codepoints.map { |point| point.to_s(16) }, four_bytes.length

text = "開 \u0080 ￿ \u{10000} \u{10FFFF} €"
spelled = text.encode("GB18030")
p spelled.bytes, spelled.encode("UTF-8") == text

p "\x80".dup.force_encoding("GB18030").valid_encoding?
p "\x81\x30".dup.force_encoding("GB18030").valid_encoding?
p "\x81\x30\x81".dup.force_encoding("GB18030").valid_encoding?
p "\x81\x30\x20\x30".dup.force_encoding("GB18030").valid_encoding?
p "\x81\x30\x81\x30".dup.force_encoding("GB18030").chars.length

begin
  "\x84\x31\xA5\x30".dup.force_encoding("GB18030").encode("UTF-8")
rescue => error
  p error.class, error.message, error.error_char.bytes, error.error_char.encoding
end

begin
  "a\xA2\xA0".dup.force_encoding("GBK").encode("UTF-8")
rescue => error
  p error.class, error.message
end
p "\x84\x31\xA5\x30z".dup.force_encoding("GB18030").encode("UTF-8", undef: :replace)
