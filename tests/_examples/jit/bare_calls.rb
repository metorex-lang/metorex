# A bare name that calls a method on `self`, in a loop run often enough to
# be compiled, with the method redefined, a method of the same name put on
# the object alone, and a local of the same name bound through a binding,
# between runs of the loop.
class Greeter
  def word = "hello"

  def say(times)
    said = []
    count = 0
    while count < times
      said << word
      count += 1
    end
    said.uniq
  end

  def shadowed
    before = word
    binding.local_variable_set(:word, "local")
    [before, word]
  end
end

greeter = Greeter.new
p greeter.say(100)
class Greeter
  def word = "hi"
end
p greeter.say(100)
p greeter.shadowed
p greeter.shadowed
other = Greeter.new
def other.word = "own"
p other.say(100)
p greeter.say(100)
