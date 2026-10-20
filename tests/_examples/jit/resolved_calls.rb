# Calls written without a receiver, with arguments or a block, made on
# main, which has a singleton class and singleton methods of its own.
def yields_once
  yield
end

def returns_from_block
  yields_once { return 1 }
end

def doubled(number)
  number * 2
end

def breaks_out
  yields_once { break :broke }
end

total = 0
100.times { total += returns_from_block }
p total

p 100.times.sum { |step| doubled(step) }
p 40.times.map { breaks_out }.uniq

def doubled(number)
  number * 3
end
p 40.times.map { |step| doubled(step) }.last

def self.doubled(number)
  -number
end
p 40.times.map { |step| doubled(step) }.last

define_singleton_method(:yields_once) { :singleton }
p 40.times.map { returns_from_block }.last

def self.method_missing(name, *arguments)
  [name, *arguments]
end
p 40.times.map { |step| not_written(step) }.last

class Counter
  def initialize
    @count = 0
  end

  def bump(by)
    @count += by
  end

  def run
    40.times { |step| bump(step) }
    @count
  end
end

counter = Counter.new
def counter.extra; end
p counter.run
def counter.bump(by)
  @count -= by
end
p counter.run
