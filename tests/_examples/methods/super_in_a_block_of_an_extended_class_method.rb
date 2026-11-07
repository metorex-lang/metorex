class Runner
  def self.with_handler(*args, &block)
    "base #{args.inspect} #{block.call}"
  end
end

module Wrapping
  def self.included(mod)
    super
    mod.instance_exec do
      extend(Wrapping::ClassMethods)
    end
  end
end

module Wrapping::ClassMethods
  def with_handler(*args, &block)
    holder = Object.new
    def holder.around
      yield
    end
    holder.around do
      super
    end
  end
end

class Spec < Runner
  include Wrapping
end

p Spec.with_handler(1, 2) { :block }

class NestedSpec < Spec
end

p NestedSpec.with_handler(3) { :nested }
