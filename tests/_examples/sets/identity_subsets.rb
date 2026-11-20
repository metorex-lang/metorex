# The sets classify, divide, flatten, and & build from a Set comparing by
# identity compare by identity too.
words = Set["one", "two", "four"].compare_by_identity
p words.classify { |word| word.length }.values.map(&:compare_by_identity?)
p words.divide { |word| word.length }.map(&:compare_by_identity?)
p words.divide { |left, right| left.length == right.length }.map(&:compare_by_identity?)

nested = Set[1, 2, Set[3, 4]].compare_by_identity
p nested.flatten.compare_by_identity?
nested.flatten!
p nested.compare_by_identity?
p nested.to_a

p (words & Set["one"]).compare_by_identity?
p (words & ["one"]).compare_by_identity?
p (words ^ ["one"]).compare_by_identity?
p words.dup.compare_by_identity?

# Two equal arrays stay apart in what & answers.
first = [1]
second = [1]
both = Set.new.compare_by_identity
both << first << second
p (both & [first]).size
p (both & Set[first, second].compare_by_identity).size

class Tagged < Set; end
tagged = Tagged[1, 2, 3]
p tagged.classify(&:odd?).values.map(&:class)
p tagged.divide(&:odd?).map(&:class)
p Tagged[1, Set[2]].flatten.class
