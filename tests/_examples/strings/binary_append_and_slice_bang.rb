buffer = "".b
buffer << "\xff\x00abc".b
buffer << "\xe2\x82\xac".b << "de".b
p buffer, buffer.encoding, buffer.bytesize

text = +""
text << "\xc3\xa9".b
p text, text.encoding

mixed = "ab".b
mixed.concat("\xff".b, "cd", "\x80".b)
p mixed, mixed.encoding

itself = "\xff\x01".b
itself << itself
p itself

cut = "\x00\x01\x02\x03\x04\x05".b
p cut.slice!(0, 2), cut
p cut.slice!(-2, 5), cut
p cut.slice!(2, 1), cut
p cut.slice!(3, 1), cut
p cut.slice!(-9, 1), cut
p cut.slice!(0, -1), cut
p cut.slice!(1, 0), cut
p cut.slice!(0, 2).encoding, cut

utf8 = +"héllo"
p utf8.slice!(1, 2), utf8
