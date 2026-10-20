# A method defined on a module itself reaches, through super, the method of
# the same name in a module it extends, also when the module is reopened by
# a second name for it and the method is called from the module's own body.
module Freight
end
Cargo = Freight

module Cargo
  module Naming
    def label(name)
      "shipping #{name}"
    end
  end
end

module Cargo
  module Labels
    extend ::Cargo::Naming

    def self.label(*args, &block)
      super(*args, &block).upcase
    end

    LABEL = label("crate")
  end
end

puts Freight::Labels.label("pallet")
puts Freight::Labels::LABEL
