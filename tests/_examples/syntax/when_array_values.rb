# A `when` listing Arrays compares the subject with each one, negative
# numbers and variables included, as RuboCop's Lint/RedundantRegexpQuantifiers
# does.
limit = 3
[[0, -1], [0, 1], [limit, limit], [5]].each do |quantity|
  label = case quantity
          when [0, -1] then "*"
          when [0, 1] then "?"
          when [limit, limit] then "{#{limit}}"
          when [1, 2], [5] then "pair or five"
          end
  p label
end
