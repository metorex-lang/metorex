# A singleton class hands back Kernel's methods from `instance_method`, as
# its class does.
object = Object.new
p object.singleton_class.instance_method(:private_methods).owner
p object.singleton_class.instance_method(:respond_to?).owner
p Object.instance_method(:private_methods).owner

class Ledger; end
p Ledger.new.singleton_class.instance_method(:frozen?).owner
