def check(list)
  if list.any? do |item|
    item > 5
end
    :big
  else
    :small
  end
end
p check([1, 2]), check([1, 9])
x = [1, 2].map do |n| n * 2 end
p x
unless [1].any? do |n| n > 3 end
  p :none
end
count = 0
while count < 2 do count += 1 end
p count
until count == 0 do count -= 1 end
p count
