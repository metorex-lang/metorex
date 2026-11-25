line_separator = "x   y"

p(/\xE2\x80\xA8/u.match?(line_separator))
p(/\342\200\250/.match?(line_separator))
p(/[\342\200\250]/u.match?(line_separator))
p line_separator.gsub(/(\\|\342\200\250|[\n\r"'])/u, " " => "&#x2028;")
p(/\xE2\x80\xA8/n.match?("\xE2\x80\xA8".b))
p(/\xE2\x80\xA8/n =~ "ab\xE2\x80\xA8".b)
p(/\101\x42/.match?("AB"))
