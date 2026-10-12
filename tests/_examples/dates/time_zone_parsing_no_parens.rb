require "time"

# A zone of -0000, UTC or Z reads as UTC, while +0000 names a local zone at
# UTC. An RFC 2822 date writes a time in UTC with the zone -0000.
read = Time.rfc2822 "Thu, 01 Jan 1970 00:00:00 -0000"
p read.utc?
read = Time.rfc2822 "Thu, 01 Jan 1970 00:00:00 +0000"
p read.utc?
read = Time.xmlschema "2000-01-01T00:00:00Z"
p read.utc?
read = Time.parse "2000-01-01 00:00:00 UTC"
p read.utc?
read = Time.httpdate "Thu, 01 Jan 1970 00:00:00 GMT"
p read.utc?
epoch = Time.at 0
p epoch.utc.rfc2822
shifted = Time.at(0).getlocal "+05:30"
p shifted.rfc822
