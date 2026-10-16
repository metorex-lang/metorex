# Blocks, yield, and return as a body compiles them, printed so the
# bytecode engine's answers can be compared with the tree walker's and Ruby's.

def twice
  first = yield(1)
  second = yield 2, 3
  [first, second]
end

def spread_yield
  pairs = [[1, 2]]
  yield(*pairs)
end

def without_a_block
  yield
rescue LocalJumpError => error
  error.message
end

def early(value)
  return :negative if value < 0
  return
end

def through_blocks
  [1, 2, 3].each do |number|
    [10, 20].each do |other|
      return number * other if number == 2
    end
  end
  :never
end

def from_a_lambda
  doubled = ->(value) { return value * 2 }
  [doubled.call(4), :after]
end

def numbered_and_it
  [[1, 2].map { _1 * 3 }, [4, 5].map { it + 1 }]
end

p twice { |first, second| [first, second] }
p spread_yield { |first, second| first + second }
p without_a_block
p [early(-1), early(1)]
p through_blocks
p from_a_lambda
p numbered_and_it
