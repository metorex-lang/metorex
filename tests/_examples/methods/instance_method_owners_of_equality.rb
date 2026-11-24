class Node
  def hash
    [self.class, @value].hash
  end

  def eql?(other)
    self.class == other.class && @value == other.value
  end
  alias :== :eql?

  attr_reader :value
end

class Quoted < Node
end

[Node, Quoted].each do |klass|
  eqeq = klass.instance_method(:==)
  p [klass, eqeq.owner, klass.instance_method(:eql?).owner, klass.instance_method(:hash).owner]
  p eqeq.super_method&.owner
end
p Object.instance_method(:==).super_method
