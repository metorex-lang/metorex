# A `respond_to_missing?` mixed into Object reaches Kernel's through `super`
# for every kind of object: an instance, an exception, and a module.
module Shipping
  def respond_to_missing?(name, include_private = false)
    name.to_s.start_with?("ship_") || super
  end
end

class Object
  include Shipping
end

p Object.new.respond_to?(:ship_crate)
p Object.new.respond_to?(:unload)
p RuntimeError.new("late").respond_to?(:unload)
p Comparable.respond_to?(:unload)
p Comparable.respond_to?(:ship_pallet)
