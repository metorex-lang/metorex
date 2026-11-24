module Encoder
  def to_json(*)
    "null"
  end
end

module Generator
end

class NilClass
  include Generator
end
NilClass.prepend(Encoder)

p nil.instance_of?(NilClass)
p true.instance_of?(TrueClass)
p 1.instance_of?(Integer)
p nil.kind_of?(Encoder)

class Integer
  prepend Encoder
end
p 1.instance_of?(Integer)
p :a.instance_of?(Symbol)
