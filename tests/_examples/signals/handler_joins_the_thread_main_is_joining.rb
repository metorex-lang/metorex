# A handler that joins the thread the main thread is joining leaves the
# main thread's join to answer once that thread is over.
worker = Thread.new do
  deadline = Process.clock_gettime(Process::CLOCK_MONOTONIC) + 1
  count = 0
  count += 1 while Process.clock_gettime(Process::CLOCK_MONOTONIC) < deadline
  :done
end
Signal.trap("TERM") { p [:handler, worker.join.value] }
sender = Process.spawn("sh", "-c", "sleep 0.3; kill -TERM #{Process.pid}")
p [:main, worker.join.value]
Process.wait(sender)
