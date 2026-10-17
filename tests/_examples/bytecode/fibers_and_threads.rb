# A fiber suspends partway through building an Array and a sum, and resumes
# with the elements built before the suspend still in place.
fiber = Fiber.new do |first|
  total = first
  pairs = [total, Fiber.yield(total + 1), total * 10]
  pairs.sum + Fiber.yield(pairs)
end
p fiber.resume(1)
p fiber.resume(5)
p fiber.resume(100)

# An external enumerator runs its block in a fiber.
letters = Enumerator.new do |yielder|
  yielder << "a" + "b"
  yielder << ["c", yielder.class.name.length].join
end
p [letters.next, letters.next]

# A thread passes control in the middle of an expression.
threads = [10, 20].map do |base|
  Thread.new { [base, (Thread.pass; base + 1), base + 2].sum }
end
p threads.map(&:value)
