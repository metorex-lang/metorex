# StringIO#read with a length answers bytes, and reads a long ASCII run in
# one piece.
require "stringio"

io = StringIO.new("abcdef")
p io.read(2), io.read(2, buffer = +"x"), buffer, io.pos
p io.read(10), io.read(1), io.read(0)
long = StringIO.new("\0" * 3_000_000)
p long.read(2_999_999).bytesize, long.pos, long.read(5)
