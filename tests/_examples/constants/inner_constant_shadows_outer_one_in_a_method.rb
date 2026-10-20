# A method finds the constant of the innermost module it was written in
# before a constant of the same name further out, whether that one is
# defined or only registered as an autoload.
module Freight
  autoload :Chain, File.join(__dir__, "autoload_shadowed_by_inner_constant", "outer_chain")

  module Pallets
    class Chain
      def kind = "inner"
    end
  end
end

module Freight
  module Pallets
    class Loader
      def build = Chain.new.kind
    end
  end
end

puts Freight::Pallets::Loader.new.build
p Freight.autoload?(:Chain).nil?
