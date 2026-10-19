# Readers called in loops run often enough to be compiled, with and
# without a receiver: attr_reader and Struct members, a reader named like
# a method objects answer natively, one on an Array subclass, one given to
# a single object, one redefined between runs, and calls TracePoint sees.
class Point
  attr_reader :across, :size

  def initialize(across)
    @across = across
    @size = across * 2
  end

  def doubled
    total = 0
    step = 0
    while step < 20
      total += across
      step += 1
    end
    total
  end
end

class Row < Array
  attr_reader :label

  def initialize(label)
    super()
    @label = label
  end
end

Pair = Struct.new(:left, :right)

def read_all(point, pair, row, times)
  total = 0
  step = 0
  while step < times
    total += point.across + point.size + pair.left
    step += 1
  end
  [total, row.label]
end

point = Point.new(3)
pair = Pair.new(5, 6)
row = Row.new(:first)
p read_all(point, pair, row, 50)
p point.doubled

def point.across = 100
p read_all(point, pair, row, 50)

class Point
  def size = -1
end
p read_all(Point.new(3), pair, row, 50)
p Point.new(4).doubled

calls = 0
trace = TracePoint.new(:call) { |event| calls += 1 if event.method_id == :doubled }
trace.enable { Point.new(1).doubled }
p calls
