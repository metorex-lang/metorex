require "stringio"

buffer = +""
stream = StringIO.new(buffer)
stream.puts "first"
stream.print "second"
p buffer
p stream.string.equal?(buffer)

held = +"abc"
StringIO.new(held).write("X")
p held

empty = StringIO.new
empty << "one"
p empty.string

replaced = +""
stream.string = replaced
stream.write "after"
p replaced
