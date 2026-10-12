Point = Struct.new(:x, :y) do
  def +(other) = Point.new(x + other.x, y + other.y)
end

total = [Point.new(1, 2), Point.new(3, 4)].sum(Point.new(0, 0))
puts total.inspect

case total
in { x: Integer => x, y: } if y > x
  puts "y leads by #{y - x}"
end
