require "date"

p Date._parse("Mon May 28 2012 00:00:00 GMT-0700 (PDT)")
p Date._parse("9000")
p Date._strptime("1999-12-31 12:00:00 -08", "%Y-%m-%d %H:%M:%S %:::z")
p Date._strptime("1999-12-31 12:00:00 -08:00", "%Y-%m-%d %H:%M:%S %:z")
p Date._strptime("1999-12-31 12:00:00 -08:00:00", "%Y-%m-%d %H:%M:%S %::z")
p Date._strptime("1470272280000", "%Q")
p Date._strptime("1470272280.123456789", "%s.%N")
