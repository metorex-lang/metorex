# A module prepended to a class's singleton class stands in front of the
# class's own `def self.` methods, and reaches them through `super`.
class Shipment
  def self.build(weight)
    "shipment of #{weight}"
  end
end

module Labeled
  def build(weight)
    "labeled #{super}"
  end
end
Shipment.singleton_class.prepend(Labeled)

puts Shipment.build(12)
p Shipment.singleton_class.ancestors.first(2)
