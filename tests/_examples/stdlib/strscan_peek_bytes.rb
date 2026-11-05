# StringScanner#peek answers the next bytes, which may end inside a
# character, and reads a long run in one slice.
require "strscan"

scanner = StringScanner.new("aé b")
p scanner.peek(2), scanner.peek(2).encoding
p scanner.peek(3), scanner.peek(10)
scanner.pos = 1
p scanner.peek(1)
scanner.getch
p scanner.peek(2)
p StringScanner.new("0" * 2_000_000).peek(1_999_999).bytesize
