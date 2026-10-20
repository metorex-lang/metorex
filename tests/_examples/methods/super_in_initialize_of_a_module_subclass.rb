# `super` from an `initialize` a Module subclass mixes in reaches the next
# `initialize` among the modules it includes, as for any other object.
module Named
  def initialize(name = nil)
    @name = name
  end

  def label = "double #{@name}"
end

module Doubling
  def initialize(doubled, *args)
    @doubled = doubled
    super(*args)
  end

  def doubled = @doubled
end

class ModuleDouble < Module
  include Named
  include Doubling
end

double = ModuleDouble.new(Comparable, "ledger")
puts double.label
p double.doubled
p double.is_a?(Module)
