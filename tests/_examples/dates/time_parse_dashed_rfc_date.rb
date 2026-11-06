# A date written day-month-year with dashes, as cookie expiry dates are,
# reads as that day of that year.
require "time"
parsed = Time.parse("Sun, 27-Sep-2037 00:00:00 GMT")
p parsed.year
p parsed.utc?
p Date._parse("Sun, 27-Sep-2037 00:00:00 GMT").values_at(:year, :mon, :mday)
p Date._parse("27-Sep-37").values_at(:year, :mon, :mday)
