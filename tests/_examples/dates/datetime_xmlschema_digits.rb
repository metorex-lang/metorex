# DateTime#xmlschema and #iso8601 write as many fraction digits as asked.
require "date"

moment = DateTime.new(2005, 2, 21, 10, 11, Rational(12_345_678, 1_000_000), "-05:00")
p moment.xmlschema
p moment.xmlschema(3)
p moment.iso8601(6)
p moment.jisx0301(2)
p moment.rfc3339(1)
p DateTime.new(2005, 2, 21).xmlschema(0)
[[1872, 12, 31], [1873, 1, 1], [1912, 7, 29], [1912, 7, 30], [1926, 12, 24], [1926, 12, 25],
 [1989, 1, 7], [1989, 1, 8], [2019, 4, 30], [2019, 5, 1], [2024, 3, 9]].each do |parts|
  p Date.new(*parts).jisx0301
end
