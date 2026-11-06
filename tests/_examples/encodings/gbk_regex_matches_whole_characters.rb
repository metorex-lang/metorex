# GBK pairs a lead byte with a trailing byte that can be an ASCII letter,
# and a pattern matches the pair as one character.
text = "許".encode(Encoding::GBK)
p text.bytes
p text.valid_encoding?
p text.length
p text.gsub(/[^a-zA-Z]/) { |match| match.unpack("H2" * match.bytesize).join("%") }
p text.inspect
p text.chars.map(&:bytes)
p text.encode("UTF-8")
p "\x81".force_encoding("GBK").valid_encoding?
