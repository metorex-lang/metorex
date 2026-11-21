p "\x9F".bytes
p "\x9Fa".bytes
p "\xFF\x9F".bytes
p "\x9F\x91\x8Dab".bytes
p "\x9F\x91\x8Dab".each_char.map(&:bytes)
p "\xF0\x9F\x91".each_char.map(&:bytes)
p "\xFF\xFEa\xC3".chars.map(&:bytes)
p "é\x9Fa".chars.map { |character| [character, character.encoding, character.valid_encoding?] }
