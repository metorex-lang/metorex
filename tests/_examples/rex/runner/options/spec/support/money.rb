Money = Struct.new(:cents) do
  def +(other) = Money.new(cents + other.cents)
  def to_s = format("$%.2f", cents / 100.0)
end
