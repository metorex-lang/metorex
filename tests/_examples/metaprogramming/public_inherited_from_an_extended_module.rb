module Wrapping
  def inherited(klass)
    puts "inherited #{klass}"
    super
  end
end

class Metal
  class << self
    private
      def inherited(subclass)
        super
      end
  end
end

class Controller < Metal
  extend Wrapping
end

class PostsController < Controller
end

PostsController.inherited(PostsController)
p PostsController.singleton_class.private_method_defined?(:inherited)
p PostsController.singleton_class.public_method_defined?(:inherited)
p PostsController.method(:inherited).owner
