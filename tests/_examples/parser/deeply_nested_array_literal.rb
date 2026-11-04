nested = eval("[" * 101 + "]" * 101)
depth = 0
while nested.is_a?(Array) && !nested.empty?
  depth += 1
  nested = nested.first
end
puts depth
pairs = [[1, 2 => 3], [:a => [4]]]
p pairs
