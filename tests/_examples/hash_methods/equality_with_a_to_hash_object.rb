require "set"

class Wrapper
  attr_reader :held

  def initialize(held)
    @held = held
  end

  def to_hash
    held
  end

  def ==(other)
    held == other
  end
end

p Set["b", "a"] == Set["a", "b"], { "k" => Set["b", "a"] } == { "k" => Set["a", "b"] }
p({ "k" => 1 } == Wrapper.new({ "k" => 1 }))
p Wrapper.new({ "k" => Set["x", "y"] }) == Wrapper.new({ "k" => Set["y", "x"] })
p({ "k" => 1 }.eql?(Wrapper.new({ "k" => 1 })))
p({ "k" => 1 } == Object.new)
