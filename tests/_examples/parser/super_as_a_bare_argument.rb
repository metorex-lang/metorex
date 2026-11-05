# `super(...)` can be the argument of a call written without parentheses.
class Base
  def make(value)
    [:base, value]
  end

  def make_bare(value)
    value
  end
end

class Wrapper
  def initialize(held)
    @held = held
  end

  def held = @held
end

class Child < Base
  def make(value)
    Wrapper.new super(value)
  end

  def make_bare(value)
    Array super(value)
  end
end

p Child.new.make(1).held
p Child.new.make_bare(2)
