# Two patterns with the same source and options are equal, and hash alike,
# whatever order their options were written in, as rubocop-ast compares
# `Regexp.new(content, options)` with the literal it came from.
built = Regexp.new(".+", Regexp::IGNORECASE | Regexp::EXTENDED | Regexp::MULTILINE)
written = eval("/.+/imx")
p built == written, built.eql?(written), built.hash == written.hash, { built => :found }[written]
p built == /.+/mi, built == Regexp.new(".+", "xim")
