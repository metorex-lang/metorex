require "timeout"

# A limit opened on the main thread does not reach a thread that sleeps
# while it is open.
sleeper = Thread.new do
  sleep
  :woken
end
answer = Timeout.timeout(30) do
  Thread.pass until sleeper.status == "sleep"
  sleeper.wakeup
  sleeper.value
end
p answer

# A limit opened on another thread ends only that thread's block.
limited = Thread.new do
  Timeout.timeout(0.05) { sleep }
rescue Timeout::Error => error
  error.message
end
p limited.value
