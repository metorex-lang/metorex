# Each literal and name a body compiles, printed so the bytecode engine's
# answers can be compared with the tree walker's.

class Shelf
  WIDTH = 3
  @@count = 2

  def self.count
    @@count
  end

  def self.count=(value)
    @@count = value
  end
end

def literals
  big = 123456789012345678901234567890
  [1, 2.5, big, nil, true, false, :name, "text", /a+b/i, 1..3, 1...3, __LINE__]
end

def names
  [::Shelf::WIDTH, Shelf::WIDTH, Shelf.count, __FILE__.end_with?("literals_and_names.rb"), __dir__.end_with?("bytecode")]
end

def hashes
  extra = { c: 3 }
  { a: 1, "b" => 2, **extra }
end

def interpolated
  count = 2
  "#{count} items, #{:symbol} #{nil}#{[1, 2]} #{1.5}"
end

def ranges
  first = 4
  last = 6
  (first..last).to_a
end

p literals
p names
p hashes
p interpolated
p ranges
