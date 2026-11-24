class WeakSet < ObjectSpace::WeakMap
  alias_method :to_a, :keys

  def <<(object)
    self[object] = true
  end
end

Excluded = WeakSet.new

module Filtering
  def subclasses
    super.reject { |held| Excluded.include?(held) }
  end
end

class Class
  prepend Filtering
end

class Parent; end
class Child < Parent; end
made = Class.new(Parent)
p Parent.subclasses.include?(made)
Excluded << made
p Excluded.include?(made)
p Parent.subclasses.include?(made)
p Parent.subclasses.size
