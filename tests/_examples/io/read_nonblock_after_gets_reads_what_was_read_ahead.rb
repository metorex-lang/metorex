reader, writer = IO.pipe
writer.write("b1:0\nc2\n")
p reader.read_nonblock(1)
p reader.gets
p reader.wait_readable(0)
p reader.read_nonblock(1)
p reader.gets
p reader.read_nonblock(1, exception: false)
writer.close
p reader.read_nonblock(1, exception: false)
