module Hooks
  module ClassMethods
    def define_hooks(name)
      ([self] + subclasses).each { |target| target.store_hook(name) }
    end

    def hooks = (@hooks ||= [])

    protected

    def store_hook(name)
      hooks << name
    end
  end
end

class Base
  extend Hooks::ClassMethods
end
class Child < Base; end
class Other; end
Base.define_hooks(:load)
p Base.hooks, Child.hooks
begin
  Child.store_hook(:outside)
rescue NoMethodError => error
  p error.message
end
