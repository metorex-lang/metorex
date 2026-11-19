# A \u escape fixes a pattern's encoding only when it names a character
# outside ASCII, so a pattern of ASCII escapes matches text in any encoding.
patterns = [/\A[\n\u0009 ]+\z/, /\u0080/, /[\u{9 20}]/, /[\u{41 e9}]/, /\\u00e9/]
p patterns.map { |pattern| [pattern.encoding, pattern.fixed_encoding?] }
p "\u0080 text" =~ /[\u0009 ]/
p "\xFF\t".b =~ /[\u0009 ]/
