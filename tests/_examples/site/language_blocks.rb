def each_pair_of(items)
  return enum_for(:each_pair_of, items) unless block_given?

  items.each_cons(2) { |first, second| yield first, second }
end

each_pair_of([1, 4, 9]) { |a, b| puts "#{a} -> #{b}" }
p each_pair_of(%w[a b c]).to_a

square = ->(n) { n * n }
add_one = proc { |n| n + 1 }
p [1, 2, 3].map(&square)
p (square >> add_one).call(4)
p %w[apple banana].map(&:upcase)
