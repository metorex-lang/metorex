reader, writer = IO.pipe
worker = Thread.new do
  Process.fork do
    reader.close
    sleep 0.1
    writer.puts "child #{Process.pid == Process.ppid ? "same" : "own"} process"
  end
end
pid = worker.value
writer.close
sleep 0.05
puts "main once"
puts reader.read
Process.wait(pid)
