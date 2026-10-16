# Subscripts written with a splat, read and written.

def spread_subscripts
  values = [1, 2, 3, 4]
  range = [1, 2]
  read = values[*range]
  values[*range] = :replaced
  table = { [1, 2] => :pair }
  [read, values, table[*[[1, 2]]]]
end

p spread_subscripts

def spread_among_subscripts
  values = [1, 2, 3, 4, 5]
  at = [1]
  values[*at, 2] = :both
  values
end

p spread_among_subscripts
