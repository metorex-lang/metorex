items = [1, 2, 3]
result = items.each_with_index.map do |item, index|
  later = items[(index + 1)..-1].any? do |other|
    break false if other > 2
    other.even?
  end
  [item, later]
end
p result
p [1, 2].map { |n| [5, 6].each { |m| break m * n } }
p [3, 4].each_with_index.map { |n, i| [n, i].find { |x| break :stop if x == 4; x > 10 } }
seen = []
p [1, 2, 3].any? { |n| seen << n; n == 2 }, seen
seen = []
p [1, 2, 3].all? { |n| seen << n; n < 2 }, seen
seen = []
p [1, 2, 3].none? { |n| seen << n; n == 1 }, seen
seen = []
p [1, 2, 3, 4].one? { |n| seen << n; n > 1 }, seen
