class Wrapper
  def initialize
    definitions = Module.new
    definitions.__send__(:define_method, :total) { |value| value * 2 }
    singleton = class << self
      self
    end
    singleton.__send__(:include, definitions)
    singleton.__send__(:define_method, :total) { |value| super(value) + 1 }
  end
end

p Wrapper.new.total(5)

class Plain
  def initialize
    definitions = Module.new do
      def total(value) = value * 3
    end
    singleton_class.include(definitions)
    singleton_class.class_eval do
      def total(value) = super + 1
    end
  end
end

p Plain.new.total(5)

greeting = Module.new { def hello = "hello" }
visitor = Object.new
visitor.extend(greeting)
def visitor.hello = super + " from an extended object"
p visitor.hello
