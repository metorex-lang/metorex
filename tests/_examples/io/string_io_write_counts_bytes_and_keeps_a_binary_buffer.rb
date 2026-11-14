# StringIO#write answers the bytes it wrote, which a caller that writes the
# rest of a partial write relies on, and a binary buffer stays binary.
require "stringio"
io = StringIO.new(String.new)
p io.write("“a”")
p io.write("x", "“")
p io.syswrite("“")
p (io << "“").string
p io.string.encoding
text = StringIO.new
p text.write("“b”")
p text.string, text.string.encoding
