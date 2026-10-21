Point = Struct.new(:x, :y)
point = Point.new(1, 2)
p point.instance_variables
point.instance_variable_set(:@label, "origin")
p point.instance_variables
p point == Point.new(1, 2)

Reading = Data.define(:value)
reading = Reading.new(value: 3)
p reading.instance_variables

class Tagged < Struct.new(:name)
  def initialize(name, tags)
    super(name)
    @tags = tags
  end
end
tagged = Tagged.new("invoice", [1, 2])
p tagged.instance_variables
copy = tagged.dup
p copy == tagged
