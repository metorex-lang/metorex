module Plain; end
made = Module.new
p made.class
p made.extend(Plain).class
p made.class
named = Module.new
p named.extend(Module.new).class
module Concerning
  def append_features(base)
    super
  end
end
first = Module.new.extend(Concerning)
p first.class
second = Module.new.extend(Concerning)
first.module_eval { def foo; [:included]; end }
klass = Class.new
klass.class_eval { def foo; super + [:class]; end }
second.module_eval { def foo; super + [:prepended]; end }
klass.include first
klass.prepend second
p klass.new.foo
