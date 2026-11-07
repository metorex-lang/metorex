s = "a\xE4 b\xE4".dup.force_encoding("ISO-8859-1")
p s.split(" ").map(&:bytes), s.split(/ /).map(&:bytes), s.split(/\s+/).map(&:encoding), s.split("").size, s.split(/x/).map(&:bytes)
binary = "\xFF \xFE".b
p binary.split(" ").map(&:bytes), binary.split(" ").map(&:encoding)
p "héllo wörld".split(" "), "héllo wörld".split(" ").map(&:encoding)
windows = "caf\xE9 cr\xE8me".dup.force_encoding("Windows-1252")
p windows.split(/ /).map(&:bytes), windows.split(/ /).map(&:valid_encoding?)
