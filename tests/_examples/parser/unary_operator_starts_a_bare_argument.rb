class Bits
  attr_reader :value

  def initialize(value) = @value = value
  def ~ = Bits.new(~value)
  def -@ = Bits.new(-value)
  def !; :negated; end
  def same?(other) = other.value == value
  def show(other) = other.respond_to?(:value) ? other.value : other
end

bits = Bits.new(5)
p bits.same? ~bits
p bits.show ~bits
p bits.show -bits
p bits.show !bits
p bits.show ~5
