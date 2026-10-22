require "time"
p Date._parse("2012-11-08T15:43:59Z").sort.to_h
p Time.zone_offset("Z")
p Time.zone_utc?("Z") rescue p $!
t = Time.utc(2012, 11, 8, 15, 43, 59)
p t.utc?
t.localtime(0)
p t
p Time.parse("2012-11-08T15:43:59Z").utc.to_s
p Time.parse("2012-11-08T15:43:59+09:00").to_s
p Date._parse("2003-12-13T18:30:02.25Z").values_at(:sec, :sec_fraction)
p Time.parse("2003-12-13T18:30:02.25Z").usec
p DateTime.parse("2003-12-13T18:30:02.25Z").sec_fraction
