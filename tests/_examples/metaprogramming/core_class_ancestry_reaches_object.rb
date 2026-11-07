class Object
  def held_on_object = :object
end
p Integer.superclass.superclass.equal?(Object)
p Numeric.method_defined?(:held_on_object)
p Integer.method_defined?(:held_on_object)
p String.method_defined?(:held_on_object)
p Comparable.instance_of?(Module)
p Integer.instance_method(:held_on_object).owner
p Object.ancestors
p BasicObject.superclass, Object.superclass
p 5.is_a?(BasicObject), Integer.ancestors.last(3)
