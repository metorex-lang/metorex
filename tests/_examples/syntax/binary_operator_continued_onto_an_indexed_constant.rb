# A binary operator that ends a line continues the expression onto the next,
# here an index into a constant whose index subtracts a method call on a
# String, as ttfunk writes it.
WEIGHT_LOWERCASE = [64, 14, 27].freeze
width = 2
total_weight = 0
lowercase_cp = "b".ord
total_weight += width *
  WEIGHT_LOWERCASE[lowercase_cp - 'a'.ord]
p total_weight
# The same with a method call ahead of the operator, which a `*` before a
# line break does not pass a splat to.
Metrics = Struct.new(:advance_width)
metrics = Metrics.new(3)
total_weight += metrics.advance_width *
  WEIGHT_LOWERCASE[lowercase_cp - 'a'.ord]
p total_weight
