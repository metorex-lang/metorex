class Version
  include Comparable

  def initialize value
    @value = value
  end

  def to_i
    @value
  end

  def <=> other
    to_i <=> other.to_i
  end
end

# A value with no `succ` is placed by comparing it against the ends, which
# `cover?` does. `include?` would have to walk the range.
low = Version.new(100)
high = Version.new(300)
inside = Version.new(200)
above = Version.new(400)
below = Version.new(50)

exclusive = low...high
inclusive = low..high

puts exclusive.cover?(inside).inspect
puts exclusive.cover?(above).inspect
puts exclusive.cover?(below).inspect
puts exclusive.cover?(low).inspect
puts exclusive.cover?(high).inspect
puts inclusive.cover?(high).inspect
numbers = 1...5
puts numbers.include?(3).inspect
letters = "a".."c"
puts letters.include?("b").inspect
