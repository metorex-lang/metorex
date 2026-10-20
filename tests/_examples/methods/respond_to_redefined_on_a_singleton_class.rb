# A `respond_to?` an object's singleton class redefines answers every call,
# including one made from a module method, and an ObjectFormatter-style
# check that calls it.
object = Object.new
calls = []
object.singleton_class.send(:define_method, :respond_to?) do |*arguments|
  calls << arguments
  false
end

module Checking
  def self.matcher?(candidate) = candidate.respond_to?(:matches?)
end

p object.respond_to?(:to_s)
p Checking.matcher?(object)
p [object].any? { |held| held.respond_to?(:inspect) }
p calls
