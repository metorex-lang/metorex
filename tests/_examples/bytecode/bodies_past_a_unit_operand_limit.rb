# Bodies with more calls than one unit's 16-bit operands reach, as generated
# API clients have, run all the same.
calls = Array.new(70_000) { |index| "  total += String.new('x').size" }.join("\n")

eval("total = 0\n#{calls}\np total")

eval(<<~RUBY)
  def count_up(start)
    total = start
  #{calls}
    return total if total > 0
    :unreached
  end
RUBY
p count_up(5)

eval(<<~RUBY)
  module Generated
    total = 0
  #{calls}
    LAST = total
  end
RUBY
p Generated::LAST

p [1].map { |start| eval("total = start\n#{calls}\ntotal") }
