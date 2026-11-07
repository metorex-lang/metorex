class Runner
  def self.with_handler(*args)
    "base #{args.inspect}"
  end
end

module ClassMethods
  def with_handler(*args)
    super
  end
end

class Spec < Runner
  extend ClassMethods
end

class NestedSpec < Spec
end

p NestedSpec.with_handler(3)
