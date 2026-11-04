# coding: US-ASCII
# A literal escaping a byte past ASCII in a US-ASCII source is BINARY.

p "\377".encoding
p "\x41".encoding
p "a\xff#{1}".encoding
p "a#{1}".encoding
