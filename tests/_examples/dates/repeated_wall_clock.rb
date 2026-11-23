# Wall clock readings a zone shows twice, read with and without the
# daylight-saving flag.
ENV["TZ"] = "Europe/Moscow"
p Time.local(2014, 10, 26, 1, 0, 0).utc
p Time.local(0, 0, 1, 26, 10, 2014, nil, nil, true, nil).utc
p Time.local(0, 0, 1, 26, 10, 2014, nil, nil, false, nil).utc
ENV["TZ"] = "America/New_York"
p Time.local(2000, 10, 29, 1, 30).utc
p Time.local(0, 30, 1, 29, 10, 2000, nil, nil, true, nil).utc
p Time.local(0, 30, 1, 29, 10, 2000, nil, nil, false, nil).utc
p Time.local(2000, 4, 2, 2, 30).utc
ENV["TZ"] = "Australia/Lord_Howe"
p Time.local(2000, 3, 26, 1, 45).utc
p Time.local(0, 45, 1, 26, 3, 2000, nil, nil, true, nil).utc
