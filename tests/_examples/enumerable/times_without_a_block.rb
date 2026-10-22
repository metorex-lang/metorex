walk = 3.times
p walk, walk.class, walk.size, walk.to_a
p 3.times.map { |i| i + 1 }, (-2).times.size, 0.times.to_a
p 4.times.with_index(1).to_a.last, 3.times.each_slice(2).to_a
