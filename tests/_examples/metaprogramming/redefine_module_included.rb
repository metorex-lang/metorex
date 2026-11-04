p Module.private_method_defined?(:included)
p Module.private_instance_methods(false).include?(:included)
Module.send(:alias_method, :included_orig, :included)
Module.send(:remove_method, :included)
seen = []
Module.send(:define_method, :included) do |base|
  included_orig(base)
  seen << base.name
end

module Greeting; end
class Visitor
  include Greeting
end
p seen

Module.send(:remove_method, :included)
Module.send(:alias_method, :included, :included_orig)
Module.send(:remove_method, :included_orig)
class Guest
  include Greeting
end
p seen
p Module.private_method_defined?(:included)
