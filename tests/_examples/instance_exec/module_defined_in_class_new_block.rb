# A module written inside a `Class.new` block belongs to the scope the block
# was written in, and `include` and `extend` there find it by its name.
module Shipping
end

module Shipping::Packing
  Builder = Class.new do
    define_method(:crate) do
      Class.new do
        module Labeling
          def label = "labeled"
        end

        extend Labeling
        include Labeling
      end
    end
  end

  Crate = Builder.new.crate
end

p Shipping::Packing.const_defined?(:Labeling, false)
puts Shipping::Packing::Crate.label
puts Shipping::Packing::Crate.new.label
p Shipping::Packing::Builder.const_defined?(:Labeling, false)
p Object.const_defined?(:Labeling, false)
