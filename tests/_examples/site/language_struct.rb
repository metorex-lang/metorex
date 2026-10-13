Point = Struct.new(:x, :y) do
  def distance_to(other)
    Math.sqrt((x - other.x)**2 + (y - other.y)**2)
  end
end

a = Point.new(0, 0)
b = Point.new(3, 4)
p a.distance_to(b)
p b.to_a
p b == Point.new(3, 4)

Coordinate = Data.define(:latitude, :longitude)
here = Coordinate.new(latitude: 36.16, longitude: -86.78)
p here
p here.with(latitude: 40.71)
