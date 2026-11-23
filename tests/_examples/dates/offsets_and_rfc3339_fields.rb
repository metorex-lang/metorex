# The offsets DateTime reads from short spellings, and the fields
# Date._rfc3339 reads from a timestamp.
require "date"

["+0", "-5", "+09", "+0530", "+05:30", "Z", "UTC", "-0", "EST"].each do |offset|
  p DateTime.civil(2000, 1, 1, 0, 0, 0, offset).offset
rescue Date::Error => error
  p [offset, error.message]
end
p Date._rfc3339("1999-12-31T19:00:00.125-05:00")
p Date._rfc3339("1999-12-31t19:00:00Z")
p Date._rfc3339("1999-12-31 19:00:00+01:00")
p Date._rfc3339("1999-12-31")
p Date._xmlschema("1999-12-31T19:00:00.5+05:00")
p Date._xmlschema("19:00:00.25Z")
p Date._xmlschema("--12-31")
p Date._xmlschema("---31+01:00")
p Date._xmlschema("1999-12")
p Date._xmlschema("nonsense")
p Date._iso8601("1999-12-31T19:00:00.125-05:00")
p DateTime.xmlschema("1999-12-31T19:00:00+05:00")
p DateTime.rfc3339("1999-12-31T19:00:00.5+05:00")
["1999-12-31", "19991231", "1999-12-31T19:00", "1999-12-31 19:00:00Z", "19991231T190000+0500",
 "1999-365", "1999-W52-5", "--12-31", "19:00:00", "T190000.5", "1999-12-31T19:00:00,5+05"].each do |text|
  p Date._iso8601(text)
end
