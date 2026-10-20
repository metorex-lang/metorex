# A BasicObject subclass that forwards what it does not answer with
# `__send__` reaches methods a module defined with `define_method`.
module Recording
  def self.record(name)
    define_method(name) { |*arguments| "#{name} #{arguments.inspect}" }
  end

  record :and_return
end

class Chain
  include Recording
end

class Forwarder < BasicObject
  def initialize(targets)
    @targets = targets
  end

  def method_missing(*arguments, &block)
    @targets.map { |target| target.__send__(*arguments, &block) }
  end
end

p Forwarder.new([Chain.new]).and_return(:result)
