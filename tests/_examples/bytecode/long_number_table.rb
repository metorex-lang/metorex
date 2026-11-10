# A literal Array of 70,000 numbers followed by a conditional, as a Racc
# grammar's tables are, and a literal of 100 that gives a new Array each
# time it runs.
source = "table = [#{(0...70_000).map { |number| number % 1000 - 1 }.join(", ")}, nil]\n" \
         "checked = :present if defined?(String)\n" \
         "[table.size, table.first, table.last, checked]"
p eval(source)

def row
  [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
   26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48,
   49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 2.5,
   true, false, nil, -100]
end
first = row
first << :changed
p [row.size, first.size, row.equal?(row), row.last(5)]
