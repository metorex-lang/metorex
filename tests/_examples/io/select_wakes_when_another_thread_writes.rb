reader, writer = IO.pipe
writing = Thread.new do
  sleep 0.05
  writer.write "x"
end
sleeping = Thread.new { sleep 0.2 }
ready = IO.select([reader], nil, nil, 5)
p ready[0] == [reader]
p reader.read_nonblock(1)
p IO.select([reader], nil, nil, 0.05)
writing.join
sleeping.join
