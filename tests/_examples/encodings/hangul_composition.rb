# Precomposed Hangul syllables stay as they are under NFC, and conjoining
# jamo compose into them.
syllables = [0xC138, 0xACC4].pack("U*")
p syllables.unicode_normalize(:nfc).codepoints.map { |point| point.to_s(16) }
p syllables.unicode_normalize(:nfd).codepoints.map { |point| point.to_s(16) }
jamo = [0x1100, 0x1161, 0x11A8].pack("U*")
p jamo.unicode_normalize(:nfc).codepoints.map { |point| point.to_s(16) }
