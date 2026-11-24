require "delegate"

class Serialized < DelegateClass(Array)
  attr_reader :coder

  def initialize(list, coder)
    @coder = coder
    super(list)
  end
end

held = Serialized.new([1, 2], :yaml)
p held.marshal_dump
p Marshal.dump(held).bytesize
loaded = Marshal.load(Marshal.dump(held))
p [loaded.coder, loaded.size, loaded.class]
simple = SimpleDelegator.new("text")
p Marshal.load(Marshal.dump(simple)).upcase
