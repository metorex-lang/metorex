quoted = "\xE2\x98\x83 snow\\\"man".b
copy = quoted.gsub(/\\(["\\])/n, "\\1")
p copy.encoding, copy.bytes.size
p copy.force_encoding("UTF-8"), copy.valid_encoding?

in_place = "\xE2\x98\x83 snow\\\"man".b
p in_place.gsub!(/\\(["\\])/n, "\\1")&.force_encoding("UTF-8")

untouched = "\xE2\x98\x83 snow".b
p untouched.gsub!(/\\(["\\])/n, "\\1")
p untouched.force_encoding("UTF-8")

sliced = "* LIST \"\xE2\x98\x83\"".b[8, 3]
p sliced.force_encoding("UTF-8")

line = "* LIST () \"/\" \"\xE2\x98\x83\xEF\xB8\x8F&\"\r\n".b
pattern = /\G(?:(\* )|"((?:[^"\\]|\\["\\])*)")/n
position = 0
words = []
while (found = line.index(pattern, position)) && found == position
  words << [$+, $+.encoding, $+.bytesize]
  position = $~.end(0)
  break if position >= 14
  position += 1 while line.getbyte(position) == 0x20 || line.getbyte(position) == 0x28 || line.getbyte(position) == 0x29
end
p words
named = "\"\xE2\x98\x83\"".b
named =~ /"(.*)"/n
p $+.bytesize, $+.force_encoding("UTF-8"), $1.dup.force_encoding("UTF-8")
