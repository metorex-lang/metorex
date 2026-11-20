module Prepended
  def prepended_probe
  end
  public :prepended_probe
end

::Module.prepend Prepended

module SelfExtending
  extend self
end

p SelfExtending.method(:prepended_probe).owner
p SelfExtending.singleton_class.ancestors.take(5)
p SelfExtending.singleton_methods
p Module.ancestors.take(3)
