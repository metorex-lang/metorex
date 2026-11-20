states = []
outer = Fiber.new do
  middle = Fiber.new do
    inner = Fiber.new do
      Fiber.new { states << :leaf_end }.transfer
      states << :inner_after_transfer
    end
    inner.resume
    states << :middle_after_resume
  end
  middle.transfer
  states << :outer_after_transfer
end
outer.resume
p states

root = Fiber.current
fiber = Fiber.new { root.transfer }
fiber.transfer
begin
  fiber.resume
rescue FiberError => error
  p error.message
end
