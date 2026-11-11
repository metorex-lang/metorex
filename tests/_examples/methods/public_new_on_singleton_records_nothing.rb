class Ver; end
class Tool < Ver; end
Tool.singleton_class.send(:define_method, :new) { |*a| super(*a) }
Tool.singleton_class.send(:remove_method, :new)
Tool.singleton_class.send(:public, :new)
Ver.singleton_class.send(:define_method, :new) { |*a| puts "Ver stub for #{self}"; super(*a) }
p Tool.singleton_class.instance_method(:new).owner
p Tool.singleton_class.method_defined?(:new), Tool.singleton_class.public_method_defined?(:new)
p Tool.singleton_class.instance_methods(false).include?(:new)
p Tool.singleton_methods(false)
