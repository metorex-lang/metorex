lock = Mutex.new
begin
  lock.synchronize { lock.synchronize { :inner } }
rescue ThreadError => problem
  p problem.message
end
p lock.locked?
