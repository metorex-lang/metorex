# A `method_added` hook a module adds to a class through `extend` in its
# `included` hook sees each method a subclass of that class defines, as
# Thor registers commands.
module Registry
  def self.included(base)
    base.extend(ClassMethods)
  end

  module ClassMethods
    def method_added(name)
      (@registered ||= []) << name
      super
    end

    def registered = @registered
  end
end

class Base
  include Registry
end

class Tasks < Base
  def build; end
  def release; end
end

p Tasks.registered

# A hook written in a superclass's `class << self` sees a subclass's methods
# too, as test-unit's TestCase sees each test method.
class Suite
  class << self
    def method_added(name)
      (@@named ||= []) << name
      super
    end

    def named = @@named
  end
end

class LedgerSuite < Suite
  def test_totals; end
end

p Suite.named
