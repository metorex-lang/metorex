class R2 < Range; end
a = R2.new(2, 2)
p a == (2..2), (2..2) == a, a.eql?(2..2), a.include?(2)
class R3 < Range
  def initialize(x, y, z); super(x, y); end
end
b = R3.new(2, 2, 0)
p b, b == (2..2), (2..2) == b
