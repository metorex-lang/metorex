# Cycling over nothing has no elements to repeat, so its size is 0.
p [].cycle.size
p [].each_entry.cycle.size
p [1].cycle.size

# A generator yielding nothing packs that as nil.
generated = Enumerator.new do |yielder|
  yielder.yield
  yielder.yield 1, 2
end
p generated.to_a

# `uniq` on a cycle that stops keeps the first of each.
taken = [1, 2, 1].cycle.first 5
p taken.uniq
