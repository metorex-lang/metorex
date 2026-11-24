require "date"
start = Date.new(2001, 2, 3)
[1, 1.0, Rational(1), Rational(3, 2), 1.5, -1.0, 13.0, 0.0].each do |months|
  begin
    p [months, start >> months, start << months]
  rescue => error
    p [months, error.class, error.message]
  end
end
p 1.eql?(Class.new(Numeric).new)
p 1.0.eql?(1.0)
