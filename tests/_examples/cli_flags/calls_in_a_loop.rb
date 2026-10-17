# Calls made once per pass of a loop: a method with an argument, a method
# reading and writing an instance variable, and a block yielded to. The
# passes are counted from the environment.
class Counter
  def initialize
    @count = 0
  end

  def bump(by)
    @count += by
  end

  attr_reader :count
end

def identity(value) = value

def run(times)
  counter = Counter.new
  index = 0
  while index < times
    counter.bump(identity(1))
    index += 1
  end
  counter.count
end

p run(Integer(ENV.fetch("PASSES", "10")))
