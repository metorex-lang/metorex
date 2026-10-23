require "stringio"
bin = "ЁЁЁ".dup.force_encoding(Encoding::BINARY)
io = StringIO.new("".encode(Encoding::UTF_8))
io.puts "Expected: \"ЁЁЁ\""
io.puts "RuntimeError: #{bin}"
io.print "x"
io << bin
io.printf("%s\n", bin)
p io.string.encoding, io.string.bytesize
s = "abc ё"
s2 = "RuntimeError: " + bin rescue p($!)
p s2.encoding
p "#{bin}".encoding
msg = "ЁЁЁ: %s" % bin rescue p($!)
p "a\n".encoding
