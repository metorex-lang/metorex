require "bigdecimal"
low = BigDecimal("0.5")
high = BigDecimal("2.25")
range = (low..high)
p range.begin, range.include?(BigDecimal("1"))
begin
  range.bsearch { true }
rescue TypeError => error
  p error.message
end
p BigDecimal("3") / 1i
p Complex(BigDecimal("3"), 0) * 2
