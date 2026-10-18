# A method that calls one of two others by which side of an `if` its
# argument takes, called with each side in turn.
def low(x)
  x
end

def high(x)
  x + 1
end

def pick(x)
  if (x < 1)
    low(x)
  else
    high(x)
  end
end

total = 0
i = 0
while i < 100
  total += pick(i % 2)
  i += 1
end
puts total
