# Array destructuring in case/in statements

# Simple array destructuring
arr = [1, 2, 3]
case arr
  in [a, b, c]
    puts "a=#{a}, b=#{b}, c=#{c}"
end

# Array with rest pattern
numbers = [1, 2, 3, 4, 5]
case numbers
  in [first, *rest]
    puts "First: #{first}"
    puts "Rest: #{rest}"
end

# Rest in the middle
case numbers
  in [first, *middle, last]
    puts "First: #{first}, Last: #{last}"
    puts "Middle: #{middle}"
end

# Nested array patterns
data = [[1, 2], [3, 4]]
case data
  in [[a, b], [c, d]]
    sum = a + b + c + d
    puts "Sum: #{sum}"
end

# Wildcard in array patterns
data2 = [1, 2, 3, 4]
case data2
  in [1, _, _, last]
    puts "First is 1, last is #{last}"
end
