# An instance of a subclass of Module, built with a block as Module.new is,
# mixes the methods the block defines into a class that includes it, and
# into an object that it extends.
class Delegator < Module
  def initialize(greeting)
    super() do
      define_method(:greeting) { greeting }

      def shout
        greeting.upcase
      end
    end
  end
end

class Visitor
  include Delegator.new("welcome")
end

visitor = Visitor.new
p visitor.greeting
p visitor.shout
p Visitor.ancestors[1].class
p Visitor.include?(Visitor.ancestors[1])

other = Object.new
other.extend(Delegator.new("hello"))
p other.shout
