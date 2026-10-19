# Calls written on a receiver that is no built-in value, and bare names
# read as locals, in units run often enough to be compiled, then the same
# sites given built-in values.
def describe(value)
  value.to_s
end

class Box
  def to_s = "box"
end

p(40.times.map { describe(Box.new) }.uniq)
p(40.times.map { |step| describe(step) }.last)

made = 40.times.map { Object.new.class }
p made.uniq

reads = 0
40.times { reads += binding.local_variable_get(:reads).to_s.size }
p reads
