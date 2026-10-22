require "power_assert"

def assertion_message(&block)
  PowerAssert.start(block, assertion_method: __callee__) do |assertion|
    assertion.yield
    assertion.message
  end
end

puts assertion_message {
  'a'.sub(/./, 'b').sub!(/./, 'c')
}

# The same assertion inside a method made with define_method, as
# test-unit makes each test.
class Checks
  define_method(:chained) do
    assertion_message {
      'a'.sub(/./, 'b').sub!(/./, 'c')
    }
  end

  def assertion_message(&block)
    PowerAssert.start(block, assertion_method: __callee__) do |assertion|
      assertion.yield
      assertion.message
    end
  end
end
puts Checks.new.chained

# The block opening on a line a heredoc starts on, as power_assert's own
# tests write them.
def show(expected, actual)
  puts expected == actual, actual
end
show <<END.chomp, assertion_message {
  'a'.sub(/./, 'b').sub!(/./, 'c')
      |             |
      |             "c"
      "b"
END
  'a'.sub(/./, 'b').sub!(/./, 'c')
}
