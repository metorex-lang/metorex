# Instance variables written in methods run often enough to be compiled:
# added to new objects, written again, written on objects of another
# shape and on a frozen object, and named like the interpreter's own.
class Pair
  def initialize(left, right, extra)
    @extra = extra if extra
    @left = left
    @right = right
  end

  def swap
    @left, @right = @right, @left
    self
  end

  def mark
    @__seen = true
  end

  def values = [@left, @right, @extra]
end

pairs = 40.times.map { |step| Pair.new(step, -step, step.odd? ? :odd : nil) }
p pairs.last(2).map(&:values)
p pairs.last(2).map { |pair| pair.swap.values }
p pairs.first.instance_variables
pairs.first.mark
p pairs.first.instance_variables
frozen = Pair.new(1, 2, nil).freeze
begin
  frozen.swap
rescue FrozenError => error
  p error.class
end
p frozen.values
