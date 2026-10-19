# Writers called in loops run often enough to be compiled: attr_writer and
# Struct members, a private writer, a frozen receiver, a writer redefined
# and one given to a single object between runs, and calls TracePoint sees.
class Point
  attr_accessor :across, :down
  attr_writer :hidden
  private :hidden=

  def hide(value)
    self.hidden = value
    @hidden
  end
end

Pair = Struct.new(:left, :right)

def fill(point, pair, times)
  step = 0
  while step < times
    point.across = step
    point.down = -step
    pair.left = step
    step += 1
  end
end

point = Point.new
pair = Pair.new(1, 2)
fill(point, pair, 50)
p [point.across, point.down, pair.left, pair.right]
p pair.to_a
p pair.instance_variables
p point.hide(:kept)
begin
  point.hidden = 1
rescue NoMethodError => error
  p error.class
end

frozen = Point.new.freeze
begin
  fill(frozen, pair, 1)
rescue FrozenError => error
  p error.class
end

def point.down=(value)
  @down = value * 10
end
fill(point, pair, 50)
p point.down

class Point
  def across=(value)
    @across = value + 1000
  end
end
fill(Point.new.tap { |fresh| fill(fresh, pair, 50) }, pair, 50)
fresh = Point.new
fill(fresh, pair, 50)
p fresh.across

calls = 0
trace = TracePoint.new(:call) { |point| calls += 1 if point.method_id == :across= }
trace.enable { fill(Point.new, pair, 3) }
p calls
