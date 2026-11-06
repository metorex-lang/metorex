done = false
other = Thread.new { sleep 0.05; done = true }
worker = Thread.new do
  Fiber.new { sleep 0.01 until done; :woke }.resume
end
sleep 0.01 until worker.status == false || worker.status.nil?
p worker.value
p other.alive?
