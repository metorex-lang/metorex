# A method whose four additions each see a Float once, 60 calls apart.
def add_ones(first, second, third, fourth)
  [first + 1, second + 1, third + 1, fourth + 1]
end

floats_at = { 100 => 0, 160 => 1, 220 => 2, 280 => 3 }
sizes = 0
400.times do |call|
  operands = [1, 2, 3, 4]
  operands[floats_at[call]] = 0.5 if floats_at.key?(call)
  sizes += add_ones(*operands).size
end
p sizes
