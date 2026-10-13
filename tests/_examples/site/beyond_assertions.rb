assert(3.between?(1, 5))
assert_equal(4, 2 + 2)
assert_raises { Integer("four") }

begin
  assert_equal(5, [2, 2].sum)
rescue RuntimeError => failure
  puts failure.message
end
