def accumulate
  total = 0
  5.times do |index|
    total += index
    binding.local_variable_set(:total, total * 10) if index == 2
  end
  total
end

puts accumulate
