module Concerning
  def append_features(base)
    super
  end

  def prepend_features(base)
    super
  end
end
included = Module.new.extend(Concerning)
prepended = Module.new.extend(Concerning)
klass = Class.new
klass.class_eval { def initialize; @foo = []; end }
included.module_eval { def foo; @foo << :included; end }
klass.class_eval { def foo; super; @foo << :class; end }
prepended.module_eval { def foo; super; @foo << :prepended; end }
klass.include included
klass.prepend prepended
p klass.new.foo
p klass.ancestors.take(3).map { |held| [included, prepended, klass].index(held) }
