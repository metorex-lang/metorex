# A StringIO set to UTF-16BE part way through reads its lines by bytes from
# where it stands, as rexml reads a document after its byte-order mark.
require "stringio"
text = ("\ufeff<?x?>\n<m/>".encode("UTF-16BE")).force_encoding("ASCII-8BIT")
io = StringIO.new(text)
first = io.read(3)
p first.bytesize, first.encoding
io.binmode
io.set_encoding("UTF-16BE", "UTF-16BE")
sep = ">".encode("UTF-16BE")
line = io.readline(sep)
p line.bytesize, line.encoding, line.b
pending = first.byteslice(2, 1).force_encoding("UTF-16BE")
joined = pending + line
p joined.bytesize, joined.encoding, joined.valid_encoding?
p joined.encode("UTF-8")
