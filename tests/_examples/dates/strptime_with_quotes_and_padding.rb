require "time"
require "date"

format = '"%d-%b-%Y %H:%M:%S %z"'
p Time.strptime('"17-Jul-1996 02:44:25 -0700"', format)
p Time.strptime('" 7-Nov-2020 01:02:03 -0400"', format)
p DateTime.strptime('"07-Nov-2020 01:02:03 -0400"', format)
p DateTime.strptime('" 7-Nov-2020 01:02:03 -0400"', format)
p Date._strptime(" 7-Nov-2020", "%d-%b-%Y")
p Date._strptime("x 7", "x%e")
p Date.new(2020, 11, 7)
p Date.new(2020, 11, 7, Date::GREGORIAN)
p DateTime.new(2020, 11, 7, 1, 2, 3.5r, "-04:00")
p DateTime.new(2020, 11, 7, 23, 0, 0, "+09:00")
p DateTime.new(1582, 10, 15, 0, 0, 0, "+00:00", Date::ENGLAND)
