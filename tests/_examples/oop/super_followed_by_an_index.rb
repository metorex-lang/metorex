# A bracket written against a bare `super` indexes what the call answers,
# as concurrent-ruby's AtomicBoolean#to_s trims the `>` off Kernel#to_s,
# while one after a space passes an Array.
class Base
  def to_s = "#<Base>"
  def items(count = 3) = [1, 2, count]
  def wrapped(list) = list.size
end
class Child < Base
  def to_s
    format '%s value:%s>', super[0..-2], 42
  end

  def items(count = 5)
    super[2]
  end

  def wrapped(list)
    super [list, list]
  end
end
p Child.new.to_s, Child.new.items, Child.new.wrapped([1])
