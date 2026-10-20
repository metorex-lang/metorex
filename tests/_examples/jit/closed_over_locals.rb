# Blocks reading locals of the code around them.
total = 0
step_size = 2
40.times { total += step_size }
p total

counts = Array.new(3, 0)
3.times do |row|
  40.times { |column| counts[row] += column % step_size }
end
p counts

reader = -> { step_size }
values = 40.times.map do |index|
  step_size = index if index == 20
  reader.call
end
p values.uniq

shadowed = 40.times.map do |index|
  binding.local_variable_set(:step_size, -1) if index == 30
  step_size
end
p shadowed.uniq

held = method(:p)
p 40.times.map { held }.uniq.map(&:name)

def double(value) = value * 2
p 40.times.map { |index| double(index) }.last

inside = 40.times.map do |index|
  eval("step_size = 5") if index == 10
  step_size
end
p inside.uniq
