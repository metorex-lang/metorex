require "bigdecimal"
require "bigdecimal/util"
p BigDecimal(9.05, 0)
p 9.05.to_d
p BigDecimal(Rational(1, 3), 10)
p Rational(22, 7).to_d(5)
