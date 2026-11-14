# `super` in a class method a superclass wrote, running for a subclass that
# extended a module with the same method, goes on past that superclass
# rather than back into the module, as concurrent-ruby's SafeInitialization
# wraps Synchronization::Object.new.
module Safe
  def build(*args) = [:safe, *super]
end
class Root
  def self.build(*args) = [:root]
end
class Mid < Root
  def self.build(*args) = [:mid, *super]
end
class Child < Mid
  extend Safe
end
class OwnChild < Mid
  extend Safe
  def self.build(*args) = [:own_child, *super]
end
p Child.build, OwnChild.build
module Barrier
  def new(*args, &block)
    super(*args, &block)
  ensure
    puts "barrier"
  end
end
class Checked
  def self.new(*args, &block)
    made = super(*args, &block)
  ensure
    puts "checked #{made.class}"
  end
end
class Value < Checked
  extend Barrier
  def initialize(held) = @held = held
end
p Value.new(1).instance_variable_get(:@held)
