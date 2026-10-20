# A module written in a `Class.new` block that is itself written in another
# `Class.new` block belongs to the scope the outer block was written in.
module Shipping
  Outer = Class.new do
    Inner = Class.new do
      module Stamping
        def stamp = "stamped"
      end

      include Stamping
    end
  end
end

p Shipping.const_defined?(:Inner, false)
p Shipping.const_defined?(:Stamping, false)
puts Shipping::Inner.new.stamp
