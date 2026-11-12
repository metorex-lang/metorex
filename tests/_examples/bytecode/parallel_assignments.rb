# `a, b = x, y` with only locals and instance variables on the left.
first, second = 1, 2
p [first, second]

first, second = second, first
p [first, second]

answer = (left, right = :l, :r)
p answer

one, two, three = 1, 2
p [one, two, three]

only, = 5, 6
p only

class Point
  def initialize(x, y)
    @x, @y = x, y
  end

  def swap
    @x, @y = @y, @x
    self
  end

  def to_a = [@x, @y]
end
p Point.new(3, 4).swap.to_a

reader, count = -> { count }, 7
p reader.call

def counted
  @calls ||= []
  @calls << :called
  @calls.size
end
low, high = counted, counted
p [low, high]

# A default that reads an instance variable or is a literal.
class Lexer
  def initialize
    @ts = 10
    @te = 14
  end

  def range(s = @ts, e = @te, flag = nil, on = true, base = 8)
    [s, e, flag, on, base]
  end

  def unset(value = @never_set) = value
end
p Lexer.new.range
p Lexer.new.range(1)
p Lexer.new.unset
