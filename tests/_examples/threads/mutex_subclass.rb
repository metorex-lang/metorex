class Guard < Mutex; end
guard = Guard.new
guard.lock
p [guard.locked?, guard.owned?]
begin
  guard.synchronize {}
rescue ThreadError => error
  p error.message
end
guard.unlock
p guard.synchronize { :inside }
