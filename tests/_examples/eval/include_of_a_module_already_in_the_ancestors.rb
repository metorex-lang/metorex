module Base
  def greet = "base"
end

module Wrapper
  include Base
  def greet = "wrapper then #{super}"
end

module Later
  include Base
end

class Holder
  include Wrapper
  include Later
end

p Holder.ancestors.take(5)
p Holder.new.greet
p Holder.instance_method(:greet).owner
