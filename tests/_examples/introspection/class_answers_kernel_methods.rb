names = Kernel.instance_methods.sort
missing = names.reject { |name| String.respond_to?(name) }
p missing
class Ledger
  undef_method :frozen?
end
p Ledger.new.respond_to?(:frozen?), Ledger.respond_to?(:frozen?)
class << Ledger
  undef_method :frozen?
end
p Ledger.respond_to?(:frozen?)
class HookList
  p respond_to?(:singleton_class)
  p singleton_class.ancestors.include?(singleton_class)
  if respond_to?(:singleton_class) && singleton_class.ancestors.include?(singleton_class)
    def which = :first
  else
    def which = :second
  end
end
p HookList.new.which
