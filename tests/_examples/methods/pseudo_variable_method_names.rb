# Methods named __LINE__, __FILE__, and __ENCODING__, as parser's AST builder
# defines them, called with a receiver, and a parameter whose name starts
# with one of those words.
class Builder
  def __LINE__(__LINE__t)
    [:line, __LINE__t]
  end

  def __FILE__(__FILE__t)
    [:file, __FILE__t]
  end

  def __ENCODING__(__ENCODING__t)
    [:encoding, __ENCODING__t]
  end
end

builder = Builder.new
p(builder.__LINE__(3), builder.__FILE__("a.rb"), builder.__ENCODING__("UTF-8"))
p(Builder.instance_methods(false).sort)
p(__LINE__, { __LINE__: 1, __FILE__: 2 }, :__ENCODING__)
