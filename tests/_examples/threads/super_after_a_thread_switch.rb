# `super` whose argument hands the turn to another thread, which makes an
# object of the same class meanwhile. The parent's method runs on the object
# the call was made on.
class Parent
  attr_reader :held

  def initialize(held)
    @held = held
  end
end

class Child < Parent
  def initialize(held, wait)
    super(Child.slowly(held, wait))
  end

  def self.slowly(held, wait)
    sleep(0.05) if wait
    held
  end
end

other = nil
thread = Thread.new { other = Child.new(:other, false) }
mine = Child.new(:mine, true)
thread.join
p [mine.held, other.held]
