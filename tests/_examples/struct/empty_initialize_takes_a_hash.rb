# A struct whose own initialize is empty still takes the arguments that
# initialize names, and a named struct with no members prints no space.
Empty = Struct.new(nil) do
  def initialize(options = {})
  end

  def self.create(options = {})
    new({}.merge(options))
  end
end
p Empty.new({})
p Empty.create(region: "us-east-1")
One = Struct.new(:a) do
  def initialize(options = {})
  end
end
p One.new({ a: 1 })
p Struct.new(nil).new
p Struct.new(nil).new.to_a
