class Posing
  def initialize(value)
    @value = value
  end

  def is_a?(klass)
    Posing == klass || @value.is_a?(klass)
  end
  alias kind_of? is_a?

  def instance_of?(klass)
    Posing == klass || @value.instance_of?(klass)
  end

  def ==(other)
    @value == other
  end

  def hash
    @value.hash
  end
end
posing = Posing.new(1)
p 1.eql?(posing)
p 1.0.eql?(Posing.new(1.0))
p "a".eql?(Posing.new("a"))
p :a.eql?(Posing.new(:a))
p (2**70).eql?(Posing.new(2**70))
p 1 == posing
