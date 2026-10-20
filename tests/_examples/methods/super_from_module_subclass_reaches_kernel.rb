# `super` from a method a Module subclass defines over one of Kernel's, such
# as `respond_to?`, reaches Kernel's.
class ModuleDouble < Module
  def respond_to?(name, include_private = false) = name == :ledger || super
end

double = ModuleDouble.new
p double.respond_to?(:ledger)
p double.respond_to?(:name)
p double.respond_to?(:shipping_label)
