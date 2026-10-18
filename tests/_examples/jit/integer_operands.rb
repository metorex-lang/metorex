def add(left, right) = left + right

puts add(1, 2)
puts add(2**62, 2**62)
puts add(1.5, 2)
puts add("deploy ", "staging")
puts add(3, 4)

def multiply(left, right) = left * right

puts multiply(6, 7)
puts multiply(-6, 7)
puts multiply(2**40, 2**40)
puts multiply(-(2**31), 2**31)
puts multiply(1.5, 2)

def modulo(left, right) = left % right

puts modulo(17, 5)
puts modulo(-17, 5)
puts modulo(17, -5)
puts modulo(-17, -5)
puts modulo(15, 5)
puts modulo(-(2**62), -1)
begin
  modulo(17, 0)
rescue ZeroDivisionError => error
  puts error.message
end
puts modulo(7.5, 2)
