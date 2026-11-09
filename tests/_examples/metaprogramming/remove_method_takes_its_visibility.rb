context = Object.new
p context.respond_to?(:method_missing)
context.singleton_class.send(:define_method, :method_missing) { |name, *args| super(name, *args) }
p context.respond_to?(:method_missing)
context.singleton_class.send(:remove_method, :method_missing)
p context.respond_to?(:method_missing)
