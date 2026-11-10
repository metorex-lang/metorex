# A pattern built from a String under the `x` option reads `#` as a comment,
# parentheses and all, as parser checks `/#)/x`, and an error names the
# options the pattern was built with.
p Regexp.new("#)", Regexp::EXTENDED)
p "line" =~ Regexp.new("l # a comment (\nine", "x")
p((Regexp.new("a)", "x") rescue $!))
p((Regexp.new("[a", Regexp::IGNORECASE | Regexp::MULTILINE) rescue $!))
