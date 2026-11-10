# A percent literal written inside an interpolation is read whole, so a quote
# or brace it holds belongs to it, as in nokogiri's XSLT parameter quoting.
value = "it's"
p "concat('#{value.gsub("'", %q{', "'", '})}')"
p "a #{%q(y)} b"
p "a #{%w{p q}.join} b"
p "a #{%q{nested {braces}}} b"
p "nested #{"x#{%q{}"}"}"}"
total = 7
p "remainder #{total % 3}"
