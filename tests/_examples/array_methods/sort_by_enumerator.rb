# Without a block, sort_by answers an Enumerator, so `with_index` can hand
# each element's place to the block that names the sort key.
rates = %w[north south east]
p(rates.sort_by.class)
p(rates.sort_by.with_index { |rate, index| -index })
p({ south: 2, north: 1 }.sort_by.with_index { |(region, _), _index| region })
