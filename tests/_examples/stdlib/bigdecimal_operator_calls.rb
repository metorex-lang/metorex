require "bigdecimal"
p BigDecimal("1.5").to_r
p BigDecimal("-2.25").to_r
p BigDecimal("7").divmod(2)
p BigDecimal("10.5").to_i
p BigDecimal("2") ** (2**70) > 0
