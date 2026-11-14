# Mutex#sleep with a length releases the lock and waits that long, so a
# thread that takes the lock meanwhile and finishes a shorter wait comes
# first. A condition variable's signal ends a timed wait early.
log = []
lock = Mutex.new
other = Thread.new do
  sleep 0.05
  lock.synchronize { log << :other }
end
lock.synchronize do
  lock.sleep(0.5)
  log << :sleeper
end
other.join
p log

ready = ConditionVariable.new
done = false
waiter = Thread.new do
  lock.synchronize do
    ready.wait(lock, 30) until done
    :signaled
  end
end
sleep 0.05
lock.synchronize do
  done = true
  ready.signal
end
p waiter.value
