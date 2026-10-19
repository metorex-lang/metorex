# encoding: binary
# The Symbols a literal in a file written in binary makes.
def binary_symbols(times)
  made = []
  count = 0
  while count < times
    made << :"café"
    count += 1
  end
  made
end
