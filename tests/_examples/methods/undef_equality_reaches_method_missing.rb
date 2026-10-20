# A class that undefines `==` answers it through `method_missing`, whether
# it is written as an operator, sent by name, or asked by an Array comparing
# its elements.
class Forwarded < Struct.new(:original)
  undef ==

  def method_missing(name, *args, &block)
    original.__send__(name, *args, &block)
  end

  def respond_to_missing?(name, include_private = false)
    original.respond_to?(name, include_private)
  end
end

pallet = Object.new
forwarded = Forwarded.new(pallet)
p forwarded == pallet
p forwarded.__send__(:==, pallet)
p forwarded == Object.new
p forwarded.send(:==, forwarded)
p [forwarded] == [pallet]
