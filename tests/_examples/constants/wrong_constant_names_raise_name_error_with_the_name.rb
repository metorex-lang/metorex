# A constant name that is not one raises NameError naming it, from each
# method that takes a constant name.
holder = Module.new
[
  -> { holder.const_defined?("Foo-bar", false) },
  -> { holder.const_defined?("foo") },
  -> { holder.const_defined?(:foo) },
  -> { holder.const_defined?("String::foo") },
  -> { holder.const_get("foo") },
  -> { holder.const_get(:foo) },
  -> { holder.const_get("Foo-bar") },
  -> { holder.const_get("String::foo") },
  -> { holder.const_set(:"Foo-bar", 1) },
  -> { holder.const_set("foo", 1) },
  -> { holder.const_source_location("Foo bar") },
  -> { holder.const_source_location("String::foo") },
  -> { holder.const_source_location(:foo) },
  -> { holder.const_source_location("foo") },
  -> { holder.const_get("Foo bar") },
  -> { holder.const_defined?("Foo bar") },
  -> { holder.const_set("Foo bar", 1) },
  -> { holder.const_get(:"String::Comparable") },
  -> { holder.const_get("::foo") },
  -> { holder.const_defined?("::foo") },
  -> { holder.send(:remove_const, :foo) },
  -> { holder.class_variable_get(:foo) },
  -> { holder.instance_variable_get("foo") },
  -> { holder.autoload(:"Foo-bar", "x") }
].each do |call|
  call.call
rescue NameError => error
  p [error.class, error.name, error.message]
end
