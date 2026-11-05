# What gsub hands its block and what scan collects are written in the
# encoding of the String the pattern matched in.
latin = "é".encode(Encoding::ISO_8859_1)
latin.gsub(/[^a-z]/) { |match| p [match.encoding, match.bytes, match.unpack("H2" * match.bytesize)] }
p latin.gsub(/[^a-z]/) { |match| "%" + match.unpack1("H2").upcase }
p latin.scan(/./).map { |match| [match.encoding, match.bytes] }
p latin.scan(/(.)/).map { |(match)| match.bytes }
p "ab".gsub(/a/) { |match| match.encoding.to_s }
