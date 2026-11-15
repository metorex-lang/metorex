# In a process forked from a thread other than the main one, that thread
# waits on a condition variable another thread of the child signals, then
# sleeps until a signal's handler ends the child.
reader, writer = IO.pipe
forked = Thread.new do
  fork do
    reader.close
    lock = Mutex.new
    signaled = ConditionVariable.new
    Thread.new do
      sleep 0.1
      lock.synchronize { signaled.signal }
    end
    lock.synchronize { signaled.wait(lock) }
    writer.puts "woken"
    Signal.trap("USR1") do
      writer.puts "trapped"
      writer.close
      exit!(0)
    end
    writer.puts "ready"
    writer.flush
    sleep
  end
end
child = forked.value
writer.close
p reader.gets
p reader.gets
Process.kill("USR1", child)
p reader.gets
Process.wait(child)
p $?.success?
