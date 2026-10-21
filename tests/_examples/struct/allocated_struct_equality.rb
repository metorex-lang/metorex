Pair = Struct.new(:left, :right)
built = Pair.new(1, 2)
allocated = Pair.allocate
allocated.left = 1
allocated.right = 2
p built == allocated, allocated == built
p built.eql?(allocated), built.hash == allocated.hash
p allocated.to_a

empty = Pair.allocate
p empty.to_a, empty == Pair.new

class Labeled < Struct.new(:name)
  def initialize(name, label)
    super(name)
    @label = label
  end
end
first = Labeled.new("invoice", [1, 2])
second = Labeled.allocate
second.send("name=", "invoice")
second.instance_variable_set(:@label, [1, 2])
p Struct.instance_method(:==).bind_call(first, second)
p first.to_a, second.to_a

class Compared < Struct.new(:name)
  def initialize(name, label)
    super(name)
    @label = label
  end

  def ==(other)
    super(other) && @label == other.instance_eval { @label }
  end
end
original = Compared.new("invoice", [1, 2])
rebuilt = Compared.allocate
rebuilt.name = "invoice"
rebuilt.instance_variable_set(:@label, [1, 2])
p original == rebuilt
p Struct.instance_method(:==).bind_call(original, rebuilt)
Inner = Struct.new(:prefix, :extensions)
class Outer < Struct.new(:personal, :other); end
a = Outer.new([Inner.new("", {})], [])
b = Outer.allocate
b.personal = [Inner.new("", {})]
b.other = []
p a == b
c = Outer.new([Inner.new("", {})], [])
p a == c
d = Outer.new([Inner.new("", {}.compare_by_identity)], [])
p a == d, {} == {}.compare_by_identity
