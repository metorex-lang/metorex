reader, writer = IO.pipe
pid = fork do
  reader.close
  writer.write("a" * 70_000)
  sleep 0.05
  writer.write("b" * 70_000)
  writer.close
  exit!(0)
end
writer.close
first = reader.read(100_000)
rest = reader.read(100_000)
p [first.size, first.count("b"), rest.size, reader.read(10)]
Process.wait(pid)
