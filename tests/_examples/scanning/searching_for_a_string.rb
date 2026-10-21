require "strscan"
s = StringScanner.new("abc literal def"); p s.scan_until("literal"), s.pos, s.matched, s.pre_match, s[0]; t = StringScanner.new("x"); p t.scan_until("y"), t.exist?("x"), t.check_until("x"), t.skip_until("x"), t.search_full("x", false, true); u = StringScanner.new("a.b"); p u.scan_until("."), u.pos
