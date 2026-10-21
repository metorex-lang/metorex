main_fiber = Fiber.current
other_fiber = Thread.new { Fiber.current }.value
puts main_fiber.equal?(other_fiber)
puts main_fiber.equal?(Fiber.current)
puts Thread.new { Fiber.current.equal?(Fiber.current) }.value
