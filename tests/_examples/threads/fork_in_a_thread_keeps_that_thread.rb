reader, writer = IO.pipe
answer_reader, answer_writer = IO.pipe
worker = Thread.new do
  pid = Process.fork do
    writer.close
    answer_reader.close
    line = reader.gets
    answer_writer.puts "child read #{line.chomp}"
    answer_writer.close
  end
  pid
end
pid = worker.value
reader.close
answer_writer.close
sleep 0.2
writer.puts "ready"
writer.close
puts answer_reader.gets
Process.wait(pid)
p $?.success?
