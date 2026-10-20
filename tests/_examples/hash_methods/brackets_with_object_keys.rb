# Hash[] keeps each key it is given as the object it is, from an array of
# pairs, from a list of keys and values, and from another hash.
Crate = Struct.new(:label) do
  def inspect = "(crate #{label})"
end

first = Crate.new("north")
second = Crate.new("south")
pairs = Hash[[[first, 1], [second, 2]]]
listed = Hash[first, 3]
copied = Hash[pairs]

p pairs
p pairs.keys.map(&:class)
p listed.keys.first.equal?(first)
p copied[second]
p copied.keys.first.equal?(first)
