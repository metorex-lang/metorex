worker = Thread.new { sleep }
sleep 0.01

pid = fork do
  queue = Queue.new
  Thread.new { sleep 0.05; queue << :woke }
  puts "child got #{queue.pop}"
end
Process.wait(pid)
p $?.exitstatus
worker.kill
