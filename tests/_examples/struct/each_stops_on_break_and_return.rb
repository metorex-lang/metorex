Pair = Struct.new(:is_truncated, :contents)
pair = Pair.new(false, [])
p pair.first, pair.first(1), pair.include?([])
answer = pair.each { |value| break :stopped }
p answer
answer = pair.each_pair { |name, value| break name }
p answer
def first_times_ten(held) = held.each { |value| return value * 10 }
p first_times_ten(Struct.new(:a, :b).new(1, 2))
