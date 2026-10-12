require "stringio"
require "strscan"

# Characters put back replace the bytes before the cursor, and what does not
# fit in front of it is added to the start.
io = StringIO.new +"1234"
io.ungetc "A"
p io.string
io = StringIO.new +"1234"
io.pos = 2
io.ungetc "φ"
p [io.pos, io.string]
io = StringIO.new +"1234"
io.pos = 2
io.ungetc "foo"
p [io.pos, io.string]

# A character read and put back is read again whole.
io = StringIO.new +"∂φ/"
letter = io.getc
io.ungetc letter
first = io.readpartial 3
p first.bytes.size
p io.read

# The rest a scanner has left is counted in bytes.
scanner = StringScanner.new "φx"
p scanner.rest_size
