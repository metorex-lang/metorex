# A Set comparing by identity keeps that through the operators that build a
# new Set from it.
first = "rate"
second = "rate".dup
tracked = Set.new([first]).compare_by_identity
joined = tracked.union [second]
p joined.size
p joined.compare_by_identity?
removed = tracked - [second]
p removed.compare_by_identity?
exclusive = tracked ^ Set[second]
p exclusive.compare_by_identity?

# `replace` takes the identity setting of the Set it copies, and `map!`
# goes back to comparing by value.
plain = Set.new [1]
plain.replace tracked
p plain.compare_by_identity?
tracked.map! { |held| held.upcase }
p tracked.compare_by_identity?

# An operand that cannot be walked is refused with Ruby's message.
begin
  Set[1] | 5
rescue ArgumentError => problem
  p problem.message
end

# The block given to `new` is applied to each element of the seed.
scaled = Set.new([1, 2]) { |held| held * 10 }
p scaled

# A Set holds classes as well as values.
classes = Set[String, Integer, String]
p classes.size
