p 1.quo(2r)
p 0.quo(2r)
p 6.quo(2**70)
p 3.quo(Complex(1, 1))
p 7.quo(2)
p 7.quo(2.0)
begin
  1.quo(0)
rescue ZeroDivisionError => error
  p error.message
end
begin
  1.quo("2")
rescue TypeError => error
  p error.message
end
p Complex(3.0, 0) / 2r
p Complex(3, 0) / 2r
p Complex(3.0, 0) * 2
p Complex(3.0, 0) / 2
p Complex(3.0, Rational(1, 2)) / 2
p Rational(1, 2) + Complex(1, 1)
p Rational(1, 2) / Complex(1, 1)
p Rational(3, 1).quo(Complex(1, 1))
p Complex(1, Rational(-1, 4))
p 3.0 / 2r
p 3.0 * 2r
p 3.0 + 2r
p 3.0 - 2r
p 2.0 ** 2r
