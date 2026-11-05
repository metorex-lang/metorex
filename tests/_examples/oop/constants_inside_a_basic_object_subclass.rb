# A method of a BasicObject subclass reads the constants its class body
# defines and the ones around it.
class ConstantHolder
  LIMIT = 3

  class Wrapper < ::BasicObject
    NAMES = [:with, :size]

    def names_include?(name, *, **)
      NAMES.include?(name)
    end

    def outer_limit
      LIMIT
    end

    def from_a_block
      [1].map { NAMES.size }
    end
  end
end

wrapper = ConstantHolder::Wrapper.new
p wrapper.names_include?(:with)
p wrapper.names_include?(:other)
p wrapper.outer_limit
p wrapper.from_a_block
