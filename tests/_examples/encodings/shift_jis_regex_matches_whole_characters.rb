# A pattern run over Shift_JIS text matches whole characters, so a
# character whose second byte is an ASCII letter is not split.
text = "ア".encode(Encoding::Shift_JIS)
p text.bytes
p text.scan(/[^a-zA-Z]/).map(&:bytes)
p text.gsub(/[^a-zA-Z]/) { |match| match.unpack("H2" * match.bytesize).join("%") }
p text.gsub(/[^a-zA-Z]/) { "x" }.encoding
