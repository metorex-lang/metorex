# `super` in each form a body compiles, printed so the bytecode engine's
# answers can be compared with the tree walker's and Ruby's.

class Base
  def greet(name, punctuation = "!", &block)
    text = "hello #{name}#{punctuation}"
    block ? block.call(text) : text
  end

  def items(*values, key: :none)
    [values, key]
  end
end

class Child < Base
  def greet(name, punctuation = "?", &block)
    [super, super(name), super(name, ".") { |text| text.upcase }, super(name, &:reverse)]
  end

  def items(*values, key: :none)
    extra = [9]
    [super(*values, *extra, key: key), super()]
  end
end

class Breaking < Base
  def greet(name)
    super(name) { |text| break :broken }
  end
end

p Child.new.greet("ann") { |text| text.length }
p Child.new.items(1, 2, key: :given)
p Breaking.new.greet("bo")
