# A pattern that opens with .* finds the same leftmost matches as one tried
# at every position, and gives up on a long subject without trying each.
p "ab\ncd\nxyz".scan(/.*y/)
p "ab\ncd\nxyz".scan(/.*y/m)
p "aXbX" =~ /.*?X/
p "line1\nline2 key\n"[/.*key/]
p "abc" =~ /(?=b).*c/
p "start --B\r\nhead\r\n\r\nbody"[/(.*?\r\n)\r\n/m, 1]
p "aaa" =~ /.*(a)\1/
p ("0123456789" * 20_000 + "\r\n") =~ /(.*?\r\n)\r\n/m
p "x\ny" =~ /.*\Ay/
