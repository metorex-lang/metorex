Top = Struct.new(:x)
p Top.name
module M
  InModule = Struct.new(:x)
end
p M::InModule.name
class C
  InClass = Struct.new(:x)
  Plain = Class.new
end
p C::InClass.name, C::Plain.name
module N
  class Deep
    Inner = Struct.new(:x)
  end
end
p N::Deep::Inner.name
