# A program whose TERM handler raises SignalException by name ends by that
# signal, with nothing reported, also when the handler runs while the main
# thread waits on a queue, on a mutex, or in another thread's join.
command = ->(body) { [RbConfig.ruby, "-e", body] }
bodies = [
  'Signal.trap("TERM") { raise SignalException, "SIGTERM" }; Process.kill("TERM", $$); sleep 5',
  'Signal.trap("TERM") { raise SignalException, "SIGTERM" }; Thread.new { sleep 0.2; Process.kill("TERM", $$) }; Queue.new.pop',
  'Signal.trap("TERM") { raise SignalException, "TERM" }; Thread.new { sleep 0.2; Process.kill("TERM", $$) }.join; sleep 5',
  'Signal.trap("TERM") { raise SignalException, "SIGTERM" }; Thread.new { sleep 5 }.tap { Process.kill("TERM", $$) }.join',
  'lock = Mutex.new; Signal.trap("TERM") { raise SignalException, "SIGTERM" }; Thread.new { lock.synchronize { sleep 0.3; Process.kill("TERM", $$); sleep 5 } }; sleep 0.1; lock.lock'
]
bodies.each do |body|
  reader, writer = IO.pipe
  pid = Process.spawn(*command.(body), err: writer, out: writer)
  writer.close
  Process.wait(pid)
  p [$?.signaled?, $?.termsig, reader.read]
end
