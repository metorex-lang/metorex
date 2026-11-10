# Patterns Onigmo accepts that regexp_parser's specs write: an option group
# with repeated dashes, a numbered backreference with a nesting level, and
# a character class intersection with an empty side.
p(/(?--m--mx--) . /mx.options, "a b" =~ /(?--m--mx--) . /mx, " q " =~ /(?--m--mx--) . /mx)
p("abcabc" =~ /(abc)\k<1-0>/, $~.to_a)
p("&" =~ /[&&]/, "a" =~ /[a&&]/, "a" =~ /[&&a]/, "b" =~ /[a-z&&b]/)
