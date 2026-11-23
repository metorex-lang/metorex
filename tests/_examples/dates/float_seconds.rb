require "date"

p DateTime.civil(2005, 2, 1, 15, 15, 10.7).sec_fraction
p DateTime.civil(2005, 2, 1, 15, 15, 10.123456789123).sec_fraction
p DateTime.civil(2005, 2, 1, 15, 15, 10.9999999999).sec_fraction
p DateTime.civil(2005, 2, 1, 15, 15, 10.0000000004).sec_fraction
p DateTime.civil(2005, 2, 1, 15, 15.5).sec_fraction
p DateTime.civil(2005, 2, 1, 15.25).min
p DateTime.jd(2453403.3).sec_fraction
p DateTime.civil(2005, 2, 1, 15, 15, 10.7) == DateTime.civil(2005, 2, 22, 15, 15, 10.7) - 21

[
  [2001, 2, 3, 4.5],
  [2001, 2, 3, 4.5, 0],
  [2001, 2, 3, 4, 5.5],
  [2001, 2, 3, 4, 5.5, 0],
  [2001, 2, 3, 4, 5, 6.5, 0],
  [2001, 2, 3, 4, -60 - Rational(1, 2)],
  [2001, 2, 3, 4, 5, -60 - Rational(1, 2)],
  [2001, 2, 3, 24],
  [2001, 2, 3, 4, "5"],
  [2001, 2, 3.5],
  [2001, 2, 3.5, 4],
  [1, 2, 3, 4, 5, 6, 0, Date::ITALY, 9]
].each do |arguments|
  p DateTime.civil(*arguments)
rescue ArgumentError, TypeError => error
  p [error.class, error.message]
end
p DateTime.jd(2451944.5)
p DateTime.ordinal(2001, 34.25)
p DateTime.commercial(2001, 5, 6, 4.5)
