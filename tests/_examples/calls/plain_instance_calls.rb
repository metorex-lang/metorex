# A thousand calls of a method with an empty body on an instance of a class
# the program wrote, from a method of the same class, named bare and with
# `self`.
class Counter
  def blank
  end

  def bare_call
    blank
  end

  def self_call
    self.blank
  end
end

counter = Counter.new
i = 0
while i < 1_000
  counter.bare_call
  counter.self_call
  i += 1
end
puts i
