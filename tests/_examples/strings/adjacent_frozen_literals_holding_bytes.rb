# frozen_string_literal: true
a = "\xE4^\xE3\xE0v@\x8Aq\xAF\xE7y\x10\x18\xD4X"\
  "\xC2\xDC&\xF6\xDB\xCCM\x03\xAF3DD\xFF\xDA\x0Flj"
p a.bytesize, a.encoding, a.valid_encoding?
b = "\xE4^\xE3\xE0v@\x8Aq\xAF\xE7y\x10\x18\xD4X"
c = "\xC2\xDC&\xF6\xDB\xCCM\x03\xAF3DD\xFF\xDA\x0Flj"
p b.bytesize, c.bytesize, (b + c).bytesize, "#{b}#{c}".bytesize
d = "\xE4" "\xE3"
p d.bytesize, d.bytes
e = "ab" "\xC2\xDC"
p e.bytesize
f = "plain " "\xE4"
p f.bytesize, f.frozen?
g = "\xE4" "plain"
p g.bytesize, g.frozen?
