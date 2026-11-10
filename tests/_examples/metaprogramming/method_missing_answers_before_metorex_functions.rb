# A receiver whose own method_missing answers a name is asked before
# metorex's `parse` and `assert` functions, which Ruby's Kernel has no method
# for, as an RSpec matcher forwards `parse(code)` to its example's helper.
class Forwarder
  def initialize(target)
    @target = target
  end

  def method_missing(name, *arguments)
    @target.respond_to?(name) ? @target.public_send(name, *arguments) : super
  end

  def respond_to_missing?(name, include_private = false)
    @target.respond_to?(name) || super
  end

  def check(code)
    [parse(code), assert(code)]
  end
end

class Example
  def parse(code) = "parsed #{code}"
  def assert(code) = "asserted #{code}"
end

p Forwarder.new(Example.new).check("[1, 2]")
p parse("1 + 2").class
