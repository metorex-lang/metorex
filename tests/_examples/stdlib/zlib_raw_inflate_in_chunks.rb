require "zlib"
require "stringio"

text = (1..4000).map { |number| "line #{number} of the ledger\n" }.join
deflater = Zlib::Deflate.new(Zlib::DEFAULT_COMPRESSION, -Zlib::MAX_WBITS)
raw = deflater.deflate(text) + deflater.finish
input = StringIO.new(raw + "trailing bytes after the stream")

inflater = Zlib::Inflate.new(-Zlib::MAX_WBITS)
output = +""
rounds = 0
until inflater.finished? || rounds > 100
  output << inflater.inflate(input.read(1000))
  rounds += 1
end
p inflater.finished?, output == text, rounds
p inflater.total_in == raw.bytesize, inflater.total_out
p inflater.inflate(nil)

finished = Zlib::Inflate.new(-Zlib::MAX_WBITS)
p finished.inflate(raw).bytesize, finished.finished?

deflater = Zlib::Deflate.new(Zlib::DEFAULT_COMPRESSION, -Zlib::MAX_WBITS)
raw = deflater.deflate("abc" * 100) + deflater.finish
one = Zlib::Inflate.new(-Zlib::MAX_WBITS)
p one.inflate(raw + "TAIL").bytesize, one.finished?, one.total_in, raw.bytesize
p one.inflate("MORE"), one.inflate(nil), one.inflate(nil)
two = Zlib::Inflate.new(-Zlib::MAX_WBITS)
p two.inflate(raw.byteslice(0, 5)).bytesize, two.total_in, two.finished?
p two.inflate(raw.byteslice(5..) + "XY").bytesize, two.total_in, two.finished?, two.total_out
p two.inflate(nil), two.finish
three = Zlib::Inflate.new
z = Zlib::Deflate.deflate("hello")
p three.inflate(z + "rest"), three.finished?, three.total_in, three.inflate(nil)
