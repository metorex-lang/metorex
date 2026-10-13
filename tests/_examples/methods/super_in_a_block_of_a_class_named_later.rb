class Reentrant
  def synchronize
    yield
  end
end

module Definitions
  def value = :from_module
end

Outer = Class.new
Outer.send(:include, Definitions)
Outer.send(:define_method, :value) { Reentrant.new.synchronize { super(&nil) } }
Inner = Class.new(Outer)
Inner.send(:include, Module.new)

p Inner.new.value
