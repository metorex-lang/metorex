require "stringio"

io = StringIO.new("こんにちは")
p io.size
p io.read(4), io.pos
p io.read(16 * 1024)
io.pos = 0
p io.read
