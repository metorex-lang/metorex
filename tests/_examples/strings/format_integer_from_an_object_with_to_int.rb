# `%i` and `%d` take an object that answers `to_int`, including one that
# answers it through `method_missing`.
class Count
  def to_int = 3
end

class Anything
  def method_missing(name, *arguments)
    case name
    when :to_int then 0
    when :to_a, :to_ary then nil
    else self
    end
  end
  def respond_to_missing?(name, include_private = false) = true
end

puts "%i" % Count.new
puts "%d" % [Count.new]
puts "%i" % Anything.new
puts format("%05d", Anything.new)

class Stand
  def method_missing(name, *arguments)
    case name
    when :to_int then 7
    when :to_a, :to_ary then nil
    else self
    end
  end

  def respond_to?(name, include_private = false) = true
end
puts "%i" % Stand.new
