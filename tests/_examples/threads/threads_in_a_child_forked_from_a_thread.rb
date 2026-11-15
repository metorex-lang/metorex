# In a process forked from a thread other than the main one, threads the
# child starts all run while it joins one of them.
reader, writer = IO.pipe
forked = Thread.new do
  fork do
    reader.close
    pinger = Thread.new do
      3.times do
        writer.puts "ping"
        writer.flush
        sleep 0.1
      end
    end
    sleeper = Thread.new { sleep 0.5 }
    sleeper.join
    pinger.join
    writer.puts "done"
    writer.close
  end
end
child = forked.value
writer.close
p reader.readlines
Process.wait(child)
p $?.success?
