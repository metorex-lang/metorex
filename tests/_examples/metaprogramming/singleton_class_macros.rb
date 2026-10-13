module Macros
  def match(&block)
    define_method(:matches?) { |actual| block.call(actual) }
  end
end

class Matcher
  extend Macros

  def initialize(declarations)
    klass = class << self
      self
    end
    klass.class_exec(&declarations)
  end
end

odd = Matcher.new(proc { match { |actual| actual.odd? } })
p odd.matches?(3)
p odd.matches?(4)
p odd.singleton_class.instance_methods(false)
p Matcher.instance_methods(false)
