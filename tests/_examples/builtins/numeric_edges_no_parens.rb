# A large Integer equals a Float only when the Float holds that exact value.
large = 2**64
p large == 18446744073709551616.0
p large + 1 == 18446744073709551616.0

# A Complex with an imaginary part is not a real number for `rect` and
# `polar`.
begin
  Complex.rect Complex(1, 2), 3
rescue TypeError => problem
  p problem.message
end
p Complex.polar 0, 1.5
p Complex.polar 2, Math::PI

# An index too large for a long is refused by name.
pair = [1, 2]
begin
  pair[large]
rescue RangeError => problem
  p problem.message
end
begin
  pair[large..]
rescue RangeError => problem
  p problem.message
end

# A hash comparing by identity equals another only when both are empty.
empty = {}.compare_by_identity
p empty == {}
held = { a: 1 }.compare_by_identity
p held == { a: 1 }

# A frozen Hash refuses a new entry.
frozen = {}.freeze
begin
  frozen[:a] = 1
rescue FrozenError => problem
  p problem.class
end
