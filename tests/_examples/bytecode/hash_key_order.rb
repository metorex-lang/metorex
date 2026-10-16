# When a Hash literal calls each key's hash, relative to evaluating the
# values written after it.

$events = []

class Key
  def initialize(name)
    @name = name
  end

  def hash
    $events << "hash #{@name}"
    @name.hash
  end

  def eql?(other)
    other.is_a?(Key) && other.name == @name
  end

  attr_reader :name
end

def value(name)
  $events << "value #{name}"
  name
end

def build
  first = Key.new(:first)
  second = Key.new(:second)
  { first => self.value(1), second => self.value(2) }
end

build
p $events
