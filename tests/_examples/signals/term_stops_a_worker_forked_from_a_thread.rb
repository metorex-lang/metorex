# A worker forked from a thread other than the main one, running several
# threads of its own, stops when its TERM handler asks it to.
reader, writer = IO.pipe
forked = Thread.new do
  fork do
    reader.close
    restart = Queue.new << true
    stop_reader, stop_writer = IO.pipe
    lock = Mutex.new
    ready = ConditionVariable.new
    stopping = false
    pool = 5.times.map do
      Thread.new do
        lock.synchronize { ready.wait(lock, 0.05) until stopping }
      end
    end
    Signal.trap("TERM") do
      restart.clear
      stop_writer.write("!")
      restart << false
    end
    stat = Thread.new { loop { writer.puts "ping"; writer.flush; sleep 0.05 } }
    while restart.pop
      server = Thread.new do
        loop do
          ready_ios, = IO.select([stop_reader], nil, nil, 0.02)
          break if ready_ios
        end
        lock.synchronize { stopping = true }
      end
      server.join
    end
    pool.each(&:join)
    stat.kill
    writer.puts "stopped"
    writer.flush
  end
end
child = forked.value
writer.close
p reader.gets
Process.kill("TERM", child)
lines = reader.readlines
p lines.last
Process.wait(child)
p $?.success?
