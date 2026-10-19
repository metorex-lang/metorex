# Calls on built-in values that reach their native methods, in a loop run
# often enough to be compiled, with a method of the same name put on one
# value alone, and then written on its class, between runs of the loop.
def sizes(values, times)
  seen = []
  count = 0
  while count < times
    seen << values.size
    count += 1
  end
  seen.uniq
end

plain = [1, 2, 3]
p sizes(plain, 100)
own = [4, 5]
def own.size = 99
p sizes(own, 100)
p sizes(plain, 100)
class Array
  def size = -1
end
p sizes(plain, 100)
p sizes("text", 100)
