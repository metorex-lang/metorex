# case/when and case/in statements as a body compiles them, printed so the
# bytecode engine's answers can be compared with the tree walker's and Ruby's.

class Point
  attr_reader :x, :y

  def initialize(x, y)
    @x = x
    @y = y
  end

  def deconstruct
    $calls << :deconstruct
    [x, y]
  end

  def deconstruct_keys(keys)
    $calls << :deconstruct_keys
    { x: x, y: y }
  end
end

def classify(value)
  case value
  when Integer, Float
    :number
  when "a".."m"
    :early
  when /z/
    :zed
  else
    :other
  end
end

def shapes(value)
  limit = 5
  case value
  in [Integer => first, *rest] if first > 0
    [:array, first, rest]
  in [*, :needle, *post]
    [:find, post]
  in { name: String => name, age: ^limit }
    [:hash, name]
  in Point(x, y)
    [:point, x, y]
  in :one | :two => either
    [:alternative, either]
  in nil
    :nothing
  end
end

def unmatched
  case 7
  in String
    :never
  end
rescue NoMatchingPatternError => error
  [error.class, error.message]
end

$calls = []
p [classify(1), classify(2.5), classify("c"), classify("zz"), classify(:x)]
p shapes([3, 4, 5])
p shapes([1, :needle, :after])
p shapes([0, :needle, :after])
p shapes({ name: "ann", age: 5 })
p shapes(Point.new(1, 2))
p shapes(:two)
p shapes(nil)
p unmatched
p $calls
