# A method a `class_exec` block defines reads constants through the modules
# the block was written in, whichever scope calls the method that runs it.
module Freight
  module Matchers
    Receive = "the Freight matcher"
  end

  module ExampleMethods
  end

  module Syntax
    def self.enable(host = ::Freight::ExampleMethods)
      host.class_exec do
        def receive = Matchers::Receive
      end
    end
  end
end

Freight::Syntax.enable
puts Object.new.extend(Freight::ExampleMethods).receive

class Configuration
  def reset = Freight::Syntax.enable
end
Configuration.new.reset
puts Object.new.extend(Freight::ExampleMethods).receive

host = Class.new
Freight::Syntax.enable(host)
puts host.new.receive

class Pallet
  SIZE = "pallet size"
end
module Freight
  SIZE = "freight size"
  WEIGHT = "freight weight"
  def self.extend_pallet
    Pallet.class_exec do
      def size = SIZE
      def weight = WEIGHT
    end
  end
end
Freight.extend_pallet
puts Pallet.new.size
puts Pallet.new.weight
