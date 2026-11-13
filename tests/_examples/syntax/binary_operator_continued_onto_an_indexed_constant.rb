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
