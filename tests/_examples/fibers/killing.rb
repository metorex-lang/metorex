done_fiber = Fiber.new { Fiber.yield }
done_fiber.resume
p done_fiber.kill.equal?(done_fiber)
p done_fiber.kill
p done_fiber.alive?

raising = Fiber.new do
  Fiber.yield
ensure
  raise "from ensure"
end
raising.resume
begin
  raising.kill
rescue RuntimeError => error
  p error.message
end

other = Fiber.new { :other }
handing_on = Fiber.new do
  Fiber.yield
ensure
  other.transfer
end
handing_on.resume
handing_on.kill
p [handing_on.alive?, other.alive?]

states = []
runner = Fiber.new do
  Fiber.new { Fiber.current.kill; states << :unreachable }.transfer
  states << :runner_resumed
end
runner.resume
p states

states = []
outer = Fiber.new do
  Fiber.new do
    outer.kill
    states << :inner_resumed
  end.transfer
  states << :outer_resumed
end
outer.resume
p states
p outer.alive?

fiber = Fiber.new { Fiber.yield }
fiber.resume
Thread.new do
  fiber.kill
rescue FiberError => error
  p error.message
end.join
