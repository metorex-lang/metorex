# Including a module that brings in one a superclass already has leaves that
# one where the superclass has it, so a method the classes between define
# still comes first, as concurrent-ruby's RubySingleThreadExecutor finds
# RubyExecutorService#post ahead of ExecutorService#post.
module Service
  def post = :abstract
end
module Serial
  include Service
end
class Abstract
  include Service
end
class Real < Abstract
  def post = :real
end
class Single < Real
  include Serial
  include Comparable
end
p Single.ancestors.take(6)
p Single.new.post
p Single.include?(Service), Single.included_modules.take(4)
