# Constant reads answered from each reference's cache, and the changes after
# which a reference looks its constant up again.
LIMIT = 1

def limits(times)
  results = []
  times.times { results << LIMIT }
  results.uniq
end

# The constant changes and the constant lookups the block made.
def counted
  before = RubyVM.stat.values_at(:constant_cache_invalidations, :constant_cache_misses).map(&:to_i)
  yield
  after = RubyVM.stat.values_at(:constant_cache_invalidations, :constant_cache_misses).map(&:to_i)
  after.zip(before).map { |late, early| late - early }
end

counted { limits(1) }
p counted { limits(100) } == counted { limits(200) }
p counted { Object.send(:remove_const, :LIMIT) }.first.positive?
Object.const_set(:LIMIT, 2)
p limits(3)

module Shapes
  SIDES = 0

  class Square
    def sides = SIDES
  end
end

square = Shapes::Square.new
p square.sides
Shapes::Square.const_set(:SIDES, 4)
p square.sides

module Named
  NAME = :named
end

class Thing
  def name = NAME
end

NAME = :top
thing = Thing.new
p thing.name
Thing.include(Named)
p thing.name

module Settings
  autoload :Later, File.join(__dir__, "constant_caches", "later.rb")

  def self.later = Later
end

p Settings.later
p Settings.later

class Box
  SIZE = :box
end

SIZE = :top
reader = proc { SIZE }
p Box.class_exec(&reader)
p reader.call
