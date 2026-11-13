module Macro
  def macro(name, &block)
    define_method(name, &block)
  end
end
