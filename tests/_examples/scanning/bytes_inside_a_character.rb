require "strscan"
s = StringScanner.new("∂x"); p s.get_byte, s.pos, s.matched, s.get_byte, s.rest, s.charpos; t = StringScanner.new("∂x"); t.pos = 1; p t.pos, t.peek_byte, t.scan_byte, t.pos, t.matched, t.eos?, t.rest
