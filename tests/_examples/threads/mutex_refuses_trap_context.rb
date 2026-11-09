lock = Mutex.new
trap("USR1") do
  lock.synchronize { $outcome = :locked }
rescue ThreadError => error
  $outcome = error.message
end
Process.kill("USR1", Process.pid)
p $outcome
p lock.synchronize { :outside }
