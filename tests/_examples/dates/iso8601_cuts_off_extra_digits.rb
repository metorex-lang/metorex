t = Time.utc(2024, 1, 2, 3, 4, 5, 678_900); p t.iso8601(3), t.strftime("%3N"), t.subsec, t.nsec, Time.at(0, 999_999_999, :nsec).utc.iso8601(3)
