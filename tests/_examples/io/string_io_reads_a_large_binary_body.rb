# A StringIO set to binary reads a long body in one bounded read, and the
# bytes past the bound stay for the next read.
require "stringio"
body = "a=1&" * 1_000_000
input = StringIO.new(body)
input.set_encoding(Encoding::BINARY)
taken = input.read(4_194_306)
p taken.bytesize
p taken.encoding
p input.read(10)
p taken.end_with?("\0")
p taken.split("&", 3).size
p taken.split(/& */n, 4097).size
p taken.split(/(?=a)/, 3).first(2)
