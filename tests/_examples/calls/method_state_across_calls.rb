# Calls of a method that only reads and writes an instance variable, with
# the method state read before and after them.
class Counter
  def initialize
    @count = 0
  end

  def inc
    @count += 1
  end
end

counter = Counter.new
counter.inc
before = __method_state__
3.times { counter.inc }
puts __method_state__ - before
