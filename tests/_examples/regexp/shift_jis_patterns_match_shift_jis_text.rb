# A pattern made from Shift_JIS text, escaped or not, matches that text,
# as minitest's assert_match does with a String, and escaping writes the
# answer in the source's own encoding.
greeting = "こんにちは".encode("Shift_JIS")
page = "<p>".dup.force_encoding("Shift_JIS") + greeting

escaped = Regexp.escape(greeting)
p [escaped.encoding, escaped.bytes.first(4)]
p [page =~ Regexp.new(greeting), page =~ Regexp.new(escaped), greeting =~ Regexp.new(greeting)]
p Regexp.escape("表.".encode("Shift_JIS")).bytes
p [Regexp.escape("é.".encode("ISO-8859-1")).bytes, Regexp.escape("é.".encode("ISO-8859-1")).encoding]
