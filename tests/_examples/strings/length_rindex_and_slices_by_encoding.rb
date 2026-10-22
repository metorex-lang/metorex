long = "aaaa " * 20_000
p long.length, long.rindex(" ", 70), long[0...70].length, long[71..-1].length

accented = "café au lait"
p accented.length, accented.rindex(" ", 9), accented[0...4], accented[5..-1]

binary = "caf\xC3\xA9".b
p binary.length, binary.rindex("f"), binary[3..-1], binary[3..-1].encoding

utf16 = "ab".encode("UTF-16LE")
p utf16.length, utf16[1..-1].bytes

wrapped = long.dup
parts = []
while wrapped.length > 70 && (place = wrapped.rindex(" ", 70))
  parts << wrapped[0...place]
  wrapped = wrapped[place + 1..-1]
end
parts << wrapped
p parts.size, parts.first, parts.last.length
