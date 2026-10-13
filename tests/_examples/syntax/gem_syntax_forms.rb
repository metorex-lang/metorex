module Core
  Event = Struct
end

class Basic < Core::Event.new(:entity); end
p(Basic.new(1).entity)

module Wrap
  class Inner < Core::Event.new(:x)
  end
end
p(Wrap::Inner.new(3).x)

begin
  ::Kernel::raise ::ArgumentError, "bad size"
rescue ArgumentError => error
  p(error.message)
end
p(::Kernel::Integer("12"))

module Tools
  def self.twice(value) = value * 2
end
p(Tools::twice(4))

def add(include)
  include + 1
end
p(add(1))
p([1, 2].map { |include| include * 2 })
p([3].each_with_object([]) { |extend, held| held << extend })

@line = 0
p({ line: @line += 1 })
count = 5
p({ total: count -= 2, "key" => count *= 3 })
module Contracts
  class CallableClass
    def kind = :callable
  end
end
module Aruba
  module Contracts
    class Enum < ::Contracts::CallableClass
    end
  end
end
p Aruba::Contracts::Enum.new.kind
class Top < ::Object
end
p Top.superclass
