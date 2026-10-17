# Objects that assign the same instance variables in the same order share a
# shape, and a read or write site reads the slot its name has there.
class Point
  def initialize(x, y)
    @x = x
    @y = y
  end

  def x = @x
  def y = @y

  def move(by)
    @x += by
    @y += by
    self
  end
end

class Reversed
  def initialize(x, y)
    @y = y
    @x = x
  end

  def x = @x
end

def shapes_made
  before = RubyVM.stat(:next_shape_id)
  yield
  RubyVM.stat(:next_shape_id) - before
end

def total_x(points)
  total = 0
  points.each { |point| total += point.x }
  total
end

Point.new(0, 0)
Reversed.new(0, 0)
p shapes_made { 10.times { |count| Point.new(count, count) } }
p shapes_made { Reversed.new(1, 2) }
p shapes_made { Object.new.instance_variable_set(:@first_of_its_kind, 1) }

points = Array.new(3) { |count| Point.new(count, 10) }
p total_x(points + [Reversed.new(100, 0)] + points)
p points.map { |point| point.move(1).y }

point = Point.new(1, 2)
point.instance_variable_set(:@z, 3)
point.remove_instance_variable(:@x)
p point.instance_variables
p [point.x, point.y]
point.instance_variable_set(:@x, 5)
p point.instance_variables
p [point.x, point.y]

frozen = Point.new(1, 2).freeze
p(begin
  frozen.move(1)
rescue FrozenError => error
  error.class
end)
p frozen.x
