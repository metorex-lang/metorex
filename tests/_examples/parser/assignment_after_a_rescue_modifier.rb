def run(list)
  if true
    index = list[1] - 2 rescue index = -1
  end
  index
end
p run([1, 5])
p run([1])
x = nil - 1 rescue y = 4
p x, y
