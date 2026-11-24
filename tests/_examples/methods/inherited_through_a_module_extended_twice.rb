module Naming
  private
  def inherited(child)
    super
    child.instance_variable_set(:@named, true)
  end
end
module Translation
  include Naming
end
module Schema
  private
  def inherited(child)
    super
    child.instance_variable_set(:@marked, true)
  end
end
class Base
  extend Naming
  extend Schema
  extend Translation
end
class Model < Base; end
class Sub < Model; end
p Base.singleton_class.ancestors.take(4)
p Model.method(:inherited).owner
p [Model.instance_variable_get(:@marked), Model.instance_variable_get(:@named)]
p [Sub.instance_variable_get(:@marked), Sub.instance_variable_get(:@named)]
