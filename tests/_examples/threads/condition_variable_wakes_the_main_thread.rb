mutex = Thread::Mutex.new
cond = Thread::ConditionVariable.new
replies = []
reader = Thread.new do
  mutex.synchronize { replies << 1; cond.signal }
end
mutex.synchronize do
  cond.wait(mutex, nil) until replies.size > 0
end
p replies
mutex.synchronize do
  Thread.new { mutex.synchronize { replies << 2; cond.signal } }
  cond.wait(mutex) until replies.size > 1
end
p replies
