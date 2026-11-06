# A method of an Object class reads top-level constants through its own
# lexical scope, even when it is called from the body of a class that does
# not descend from Object.
class Reader
  def kind
    Integer
  end
end

def top_level_kind
  Float
end

class Bare < BasicObject
  ::Kernel.p ::Kernel.send(:top_level_kind) rescue ::Kernel.p $!
  ::Kernel.p ::Reader.new.kind
  ::Kernel.p((/a/ =~ "xa"))
  ::Kernel.p ::Regexp.last_match.begin(0)
end
