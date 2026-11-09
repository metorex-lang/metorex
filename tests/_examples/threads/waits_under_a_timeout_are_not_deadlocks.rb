require "timeout"
gate = Queue.new
held = Thread.new { gate.pop }
Thread.pass until held.status == "sleep"
waiting = Queue.new
begin
  Timeout.timeout(0.1) { waiting.pop }
rescue Timeout::Error
  p :timed_out
end
gate << :go
held.join
p held.status
