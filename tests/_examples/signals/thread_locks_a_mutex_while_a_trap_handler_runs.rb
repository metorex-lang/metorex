# A trap handler that waits for another thread lets that thread lock a mutex.
lock = Mutex.new
locked = Queue.new
Signal.trap("USR1") do
  worker = Thread.new { lock.synchronize { locked << :locked } }
  worker.join
end
Process.kill("USR1", Process.pid)
sleep 0.1 while locked.empty?
p locked.pop
begin
  Signal.trap("USR2") { lock.lock }
  Process.kill("USR2", Process.pid)
  sleep 0.1
rescue ThreadError => error
  p error.message
end
