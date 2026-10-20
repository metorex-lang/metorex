# An instance of a Module subclass with a singleton method of its own still
# finds a method its class's modules define ahead of one BasicObject has.
class BasicObject
  def tracked? = :basic_object
end

module Tracking
  def tracked? = :tracking
end

class ModuleDouble < Module
  include Tracking
end

double = ModuleDouble.new
p double.tracked?
class << double
  define_method(:pallet_count) { 12 }
end
p double.pallet_count
p double.tracked?
p double.method(:tracked?).owner
p double.singleton_class.superclass
p double.singleton_class.ancestors.take(3).drop(1)
