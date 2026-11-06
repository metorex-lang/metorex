copy = String.new("\xE3\x81\x82".b)
reader, writer = IO.pipe
writer.write copy
writer.close
p reader.read.bytes
