# Keywords collected by a method marked with ruby2_keywords stay keywords
# when the collected arguments are handed to `new`, kept by an `initialize`
# also marked, and sent on with `__send__`, with or without a block.
class Predicate
  def initialize(name, *args, &block)
    @name = name
    @args = args
    @block = block
  end
  ruby2_keywords :initialize

  def run(target)
    target.__send__(@name, *@args, &@block)
  end
end

class Builder
  def method_missing(name, *args, &block)
    Predicate.new(name, *args, &block)
  end
  ruby2_keywords :method_missing
end

class Shipment
  def heavy?(limit: 10)
    limit > 5 && (!block_given? || yield)
  end
end

puts Builder.new.heavy?(limit: 7).run(Shipment.new)
puts Builder.new.heavy?(limit: 3).run(Shipment.new)
puts Builder.new.heavy?(limit: 7) { false }.run(Shipment.new)
