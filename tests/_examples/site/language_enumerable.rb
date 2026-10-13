orders = [
  { item: "keyboard", price: 85, quantity: 2 },
  { item: "monitor", price: 240, quantity: 1 },
  { item: "cable", price: 9, quantity: 6 },
]

p orders.sum { it[:price] * it[:quantity] }
p orders.max_by { it[:price] }[:item]
p orders.partition { it[:price] > 50 }.map(&:size)
p orders.group_by { it[:quantity] > 1 }.transform_values { |group| group.map { it[:item] } }
p (1..Float::INFINITY).lazy.select(&:even?).map { it * it }.first(3)
p %w[a b c].each_with_index.to_h
