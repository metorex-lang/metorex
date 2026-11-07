wide = "\x00h\x00i".dup.force_encoding("UTF-16BE")
whole = "=?x?=".gsub(/=\?x\?=/) { wide }
p whole.encoding, whole.bytes

prefixed = begin; "a =?x?=".gsub(/=\?x\?=/) { wide }; rescue => error; p error; end
p prefixed.encoding, prefixed.bytes if prefixed.is_a?(String)

two = "=?x?==?x?=".gsub(/=\?x\?=/) { wide }
p two.encoding, two.bytes

tagged = "=?x?=".gsub(/=\?x\?=/) { "\xA4\xA2".dup.force_encoding("EUC-JP") }
p tagged.encoding, tagged.bytes

empty = "".gsub(/x/) { wide }
p empty.encoding

plain = "=?x?=".gsub(/=\?x\?=/) { "hi".encode("US-ASCII") }
p plain.encoding
