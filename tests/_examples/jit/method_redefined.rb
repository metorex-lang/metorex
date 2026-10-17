def total(count)
  sum = 0
  index = 0
  while index < count
    sum = sum + index
    if index == 5
      Integer.class_eval do
        alias_method :original_plus, :+
        define_method(:+) { |other| original_plus(other).original_plus(1000) }
      end
    end
    index = index.succ
  end
  sum
end

puts total(8)
