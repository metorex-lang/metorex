require "date"

p Date.new(2020, 1, 2).marshal_dump
p DateTime.new(2020, 1, 2, 3, 4, 5.25r, "+02:00").marshal_dump
p Date.new(2020, 1, 2, Date::GREGORIAN).marshal_dump
p Marshal.dump(Date.new(2020, 1, 2))
p Marshal.load(Marshal.dump(Date.new(2020, 1, 2)))
p Marshal.load(Marshal.dump(DateTime.new(2020, 1, 2, 3, 4, 5.25r, "+02:00")))
loaded = Date.allocate
loaded.marshal_load([0, 2458851, 0, 0, 0, 2299161.0])
p loaded
p Marshal.load(Marshal.dump(Date.new(1500, 1, 2, Date::GREGORIAN))).julian?
