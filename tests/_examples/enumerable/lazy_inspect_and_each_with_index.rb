lazy = (1..3).lazy
p lazy
p lazy.map { _1 }
p lazy.collect { _1 }
p lazy.select { _1 }
p lazy.filter { _1 }
p lazy.find_all { _1 }
p lazy.collect_concat { _1 }
p lazy.take(2)
p lazy.take(0)
p lazy.drop(1)
p lazy.with_index
p lazy.with_index(1)
p lazy.each_with_index
p lazy.zip([4])
p lazy.zip
p lazy.each_slice(2)
p lazy.each_cons(2)
p lazy.with_object(1)
p lazy.compact
p lazy.uniq
p lazy.grep(Integer)
p lazy.grep_v(1)
p lazy.to_enum
p lazy.map { _1 }.take_while { _1 }.flat_map { _1 }

with_indexes = (1..3).lazy.each_with_index
p with_indexes.size
p with_indexes.to_a
p with_indexes.map { |value, index| value * index }.to_a
p with_indexes.first(2)
p (1..Float::INFINITY).lazy.each_with_index.map { |value, index| value + index }.first(3)
p with_indexes.with_index.to_a
p [[1, 2], [3, 4]].lazy.each_with_index.map { |(first, second), index| first + second + index }.to_a
p with_indexes.eager.to_a
p Enumerator::Lazy.instance_method(:each_with_index).owner
p Enumerator::Lazy.private_method_defined?(:_enumerable_each_with_index)

p (1..3).lazy.each_with_index { |value, index| value }
p (1..3).lazy.map { _1 * 2 }.each_with_index { |value, index| value }
p (1..3).lazy.each { }
p (1..3).lazy.map { _1 }.each { }
p (1..3).lazy.take(0).each { }
p [1].lazy.each_slice(1).each { }
p Enumerator::Lazy.new(1..3) { |yielder, value| yielder << value }.each { }
