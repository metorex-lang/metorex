class Counted < Numeric
  def initialize(value)
    @value = value
  end

  def to_i
    @value
  end
end

class Exact < Numeric
  def to_r
    Rational(1, 2)
  end

  def to_int
    0
  end
end

base = Time.at(100).utc
p base + Counted.new(-1)
p base - Counted.new(5)
p base + Exact.new
p (base + Exact.new).subsec
["1", nil, Object.new].each do |bad|
  begin
    base + bad
  rescue TypeError => error
    p error.message
  end
end
only_exact = Object.new
def only_exact.to_r
  Rational(1)
end
begin
  base + only_exact
rescue TypeError => error
  p error.message
end
