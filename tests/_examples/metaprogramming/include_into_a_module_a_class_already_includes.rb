# A module a class includes keeps its place in the class's ancestors when a
# module the class includes, or its superclass, includes it later.
module Shared
  def where = "Shared"
end
module Earlier
  def where = "Earlier"
end
module Mixed; end
class Base; end
class Derived < Base
  include Mixed
  include Earlier
  include Shared
end

Base.include(Shared)
Mixed.include(Shared)
p Derived.ancestors.take_while { |ancestor| ancestor != Object }
p Derived.new.where
p Derived.instance_method(:where).owner

Array.include(Shared)
Object.include(Shared)
Enumerable.include(Shared)
p Array.ancestors.take(3)
p [].where
