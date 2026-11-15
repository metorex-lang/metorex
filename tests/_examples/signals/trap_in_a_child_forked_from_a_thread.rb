# A process forked from a thread other than the main one runs its trap
# handlers in that thread, which is the child's only one.
reader, writer = IO.pipe
forked = Thread.new do
  fork do
    reader.close
    Signal.trap("USR1") { writer.puts "trapped"; writer.flush }
    writer.puts "ready"
    writer.flush
    deadline = Process.clock_gettime(Process::CLOCK_MONOTONIC) + 5
    sleep 0.05 until Process.clock_gettime(Process::CLOCK_MONOTONIC) > deadline
  end
end
child = forked.value
writer.close
p reader.gets
Process.kill("USR1", child)
p reader.gets
Process.kill("KILL", child)
Process.wait(child)
