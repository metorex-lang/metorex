# Bare names a `def` at the top level bound, read in methods and blocks
# run often enough to be compiled, with the method redefined between runs
# and called from a method of another class.
def foo
  yield
end

def bar
  foo { return 1 }
end

def twice
  yield
  yield
end

total = 0
30.times do
  twice { total += bar }
end
p total

def bar
  2
end
total = 0
30.times { total += bar }
p total

class Point
  def through_bar = bar
end
p(30.times.map { Point.new.through_bar }.uniq)
