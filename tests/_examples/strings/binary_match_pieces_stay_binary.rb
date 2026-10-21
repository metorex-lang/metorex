s = "a\xE2\x98\x83".b
s.index(/(.\xE2)/n); p [$~[1].encoding, $+.encoding]
s =~ /(.\xE2)/n; p [$~[1].encoding, $+.encoding]
s.match(/(.\xE2)/n); p [$~[1].encoding, $+.encoding]
s.rindex(/(.\xE2)/n); p [$~[1].encoding, $+.encoding]
s[/(.\xE2)/n]; p [$~[1].encoding, $+.encoding]
