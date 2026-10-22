require "power_assert"

def assertion_message(&block)
  PowerAssert.start(block, assertion_method: __callee__) do |assertion|
    assertion.yield
    assertion.message
  end
end

a = 1
puts assertion_message {
  ! a != (+a == -a)
}

define_method(:bmethod) { false }
puts assertion_message {
  bmethod
}

b = 0
puts assertion_message {
  String(a) + String(b)
}

puts assertion_message {
  "3".to_i.times.map { |i| i + 1 }.class
}

class Aliased
  def original
    :foo
  end
  alias_method :alias_of_original, :original

  def inspect = "#<Aliased>"
end
puts assertion_message {
  Aliased.new.alias_of_original
}
