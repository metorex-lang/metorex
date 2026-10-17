def classify(value)
  raise ArgumentError, "negative #{value}" if value.negative?

  value.even? ? :even : :odd
end

[3, 4, -1].each do |value|
  puts classify(value)
rescue ArgumentError => error
  puts error.message
ensure
  puts "checked #{value}"
end

found = [5, 8, 13, 21].each_with_index do |value, index|
  break index if value > 10
end
puts found

total = 0
index = 0
while index < 10
  index += 1
  next if index.odd?
  total += index
  break if total > 20
end
puts total

caught = catch(:stop) do
  10.times { |count| throw :stop, count * 3 if count == 4 }
  :never
end
puts caught

def deep(level) = level.zero? ? 0 : 1 + deep(level - 1)
puts deep(200)

fiber = Fiber.new do
  Fiber.yield 1
  Fiber.yield 2
  3
end
puts [fiber.resume, fiber.resume, fiber.resume].inspect
