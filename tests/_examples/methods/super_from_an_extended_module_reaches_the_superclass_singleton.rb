# A module a class extends itself with, from an included hook, calls super
# into a singleton method its superclass defined.
class Runner
  def Runner.run_named(label)
    "named #{label}"
  end

  def self.run_self(label)
    "self #{label}"
  end

  class << self
    def run_opened(label)
      "opened #{label}"
    end
  end
end

module Wrapping
  def self.included(base)
    base.extend(ClassMethods)
  end

  module ClassMethods
    def run_named(label)
      "wrapped " + super
    end

    def run_self(label)
      "wrapped " + super
    end

    def run_opened(label)
      "wrapped " + super
    end
  end
end

class WrappedRunner < Runner
  include Wrapping
end

puts WrappedRunner.run_named(1)
puts WrappedRunner.run_self(2)
puts WrappedRunner.run_opened(3)

class ExtendedRunner < Runner
  extend Wrapping::ClassMethods
end

puts ExtendedRunner.run_named(4)
