# defined? finds the public copy module_function makes, and still refuses a
# class method made private.
module Escaping
  module_function

  def escape(text) = text
end

module Hidden
  def self.helper = 1
  private_class_method :helper
end

held = Escaping
p defined?(Escaping.escape)
p defined?(held.escape)
p defined?(Hidden.helper)
p Escaping.private_methods(false).include?(:escape)
