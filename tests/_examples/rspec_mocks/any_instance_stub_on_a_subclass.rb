# A method a subclass inherits can be stubbed on any instance of the subclass.
require "rspec/mocks"

class Grandparent
  def foo(_argument) = "bar"
end

class Parent < Grandparent
  def foo = super(:a)
end

class Child < Parent
end

include RSpec::Mocks::ExampleMethods
RSpec::Mocks.setup
allow_any_instance_of(Child).to receive(:foo).and_return(:result)
p Child.new.foo
RSpec::Mocks.teardown
p Child.new.foo
