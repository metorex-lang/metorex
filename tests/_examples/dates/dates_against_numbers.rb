require "date"

p DateTime.civil(1970, 1, 1, 12) <=> 2440588
p DateTime.civil(1970) <=> 2440587.5
p DateTime.civil(1970) <=> Rational(4881177, 2)
p Date.civil(1970) <=> 2440587
p Date.civil(1970) <=> 2440587.5
p DateTime.civil(1970, 1, 1, 6, 0, 0, "+03:00").ajd
p DateTime.civil(1970, 1, 1, 6, 0, 0, "+03:00").amjd
p Date.civil(1970).ajd
p Date.civil(1970).amjd
