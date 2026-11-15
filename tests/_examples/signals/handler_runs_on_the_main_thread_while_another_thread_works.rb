# A signal another process sends while a thread is busy runs its handler on
# the main thread, so an exception it raises is not rescued by the busy thread.
Signal.trap("TERM") { raise SignalException, "SIGTERM" }
rescued_in_worker = []
worker = Thread.new do
  begin
    deadline = Process.clock_gettime(Process::CLOCK_MONOTONIC) + 3
    count = 0
    count += 1 while Process.clock_gettime(Process::CLOCK_MONOTONIC) < deadline
  rescue Exception => error
    rescued_in_worker << error.class
  end
end
sender = Process.spawn("sh", "-c", "sleep 0.3; kill -TERM #{Process.pid}")
begin
  worker.join
rescue SignalException => error
  p [:main, error.message, Thread.current == Thread.main]
end
worker.join
Process.wait(sender)
p rescued_in_worker
