require "stringio"

io = StringIO.new("éééabc")
p [io.gets(1), io.gets(3), io.gets(2), io.pos]
io = StringIO.new("👍ab")
p [io.gets(3), io.pos]
io = StringIO.new("👍ab")
io.read(1)
p [io.gets(1).bytes, io.pos]
io = StringIO.new("あいabc".encode("Shift_JIS"))
p [io.gets(1), io.gets(3)].map { |line| line.encode("UTF-8") }
io = StringIO.new("aé>b")
p [io.gets(">", 2), io.pos]
p StringIO.new("ééa").gets(nil, 3)
p StringIO.new("ééé").readlines(3)
p StringIO.new("ééé").each_line(3).to_a
p StringIO.new("éa\nb\n").each_line("\n", 3, chomp: true).to_a
p StringIO.new("a\nb\n").readlines("\n", 1, chomp: true)
