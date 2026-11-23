# Time.local in the ten-argument order Time#to_a gives: the daylight saving
# flag counts only for a time the clock passes twice.
ENV["TZ"] = "US/Central"

p Time.local(10, 10, 10, 22, 4, 2005, nil, nil, false, nil)
p Time.local(10, 10, 10, 22, 2, 2005, nil, nil, true, nil)
p Time.local(0, 30, 1, 30, 10, 2005, nil, nil, true, nil)
p Time.local(0, 30, 1, 30, 10, 2005, nil, nil, false, nil)
p Time.local(0, 30, 2, 3, 4, 2005, nil, nil, false, nil)
p Time.local(*Time.local(2005, 6, 1, 12).to_a)
p Time.local(2005, 4, 22, 10, 10, 10)
