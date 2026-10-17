# Calls answered from a call site's method cache, and the changes after which
# a call site looks its method up again.
class Point
  def initialize(x)
    @x = x
  end

  def x
    @x
  end
end

class Label
  def x
    :label
  end
end

def read_each(points)
  results = []
  index = 0
  while index < points.size
    results << points[index].x
    index += 1
  end
  results
end

# The lookups and the cached answers the block's calls made.
def counted
  before = RubyVM.stat.values_at(:method_cache_misses, :method_cache_hits).map(&:to_i)
  yield
  after = RubyVM.stat.values_at(:method_cache_misses, :method_cache_hits).map(&:to_i)
  after.zip(before).map { |late, early| late - early }
end

point = Point.new(1)
label = Label.new
read_each([point])

p counted { read_each([point] * 100) }
p counted { read_each([point, label] * 50) }
p counted { read_each([point, label, Point.new(2), Label.new] * 25) }
classes = Array.new(5) { Class.new { def x = 0 } }
p counted { read_each(classes.map(&:new) * 20) }

class Walker
  def step = 1

  def walk(times)
    total = 0
    index = 0
    while index < times
      total += step()
      index += 1
    end
    total
  end
end

walker = Walker.new
walker.walk(1)
p counted { walker.walk(100) }

Point.define_method(:x) { @x * 10 }
p read_each([point])

module Doubled
  def x
    super * 2
  end
end

Point.prepend(Doubled)
p read_each([point])

class Base
  def x
    :base
  end
end

class Child < Base
  def x
    :child
  end
end

child = Child.new
p read_each([child])
Child.remove_method(:x)
p read_each([child])

module Mixed
  def x
    :mixed
  end
end

Child.include(Mixed)
p read_each([child])

class Child
  private :x
end
p(begin
  read_each([child])
rescue NoMethodError => error
  error.message
end)

def point.x
  :own
end
p read_each([point])

module Refined
  refine Label do
    def x
      :refined
    end
  end
end

using Refined
p [label].map(&:x)
p label.x
p read_each([label])
