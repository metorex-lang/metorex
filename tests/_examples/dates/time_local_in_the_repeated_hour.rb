# A local time the clock shows twice, as at the end of daylight saving time,
# reads as the later of the two.
ENV["TZ"] = "US/Eastern"

p Time.local(2005, 10, 30, 1, 0, 0)
p Time.local(2005, 10, 30, 1, 30, 0)
p Time.local(2005, 10, 30, 0, 59, 59) + 1
p Time.new(2005, 10, 30, 1, 0, 0)
p Time.local(2005, 10, 30, 2, 0, 0)
p Time.local(2005, 4, 3, 2, 30, 0)
ENV["TZ"] = "Australia/Adelaide"
p Time.local(2005, 3, 27, 2, 30, 0)
p Time.local(2005, 3, 27, 2, 0, 0)
