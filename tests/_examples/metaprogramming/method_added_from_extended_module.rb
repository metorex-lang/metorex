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
