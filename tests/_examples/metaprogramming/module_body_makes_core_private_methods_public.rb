p Module.private_instance_methods.include?(:using)
p Module.private_instance_methods.include?(:refine)

class Module
  public :remove_const
end

p Module.public_method_defined?(:remove_const)

module Settings
  TIMEOUT = 30
end

Settings.remove_const(:TIMEOUT)
p Settings.const_defined?(:TIMEOUT)
