require "date"
d = DateTime.new(2026, 11, 6, 19, 6, 45)
p d.ctime, d.asctime, Date.new(2026, 11, 6).ctime, d.to_date.ctime
