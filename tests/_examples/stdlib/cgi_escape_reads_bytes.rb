# CGI.escape and CGI.escapeURIComponent write every byte but the unreserved
# characters in hex, including the bytes of a run that spells no character,
# and keep the encoding of the text they were given.
require "cgi"

texts = ["a b&c=d/e~f_g.h-i", "café 日本", "bad \xFF\xFE end".b, "plus+percent%", ""]
texts.each do |text|
  escaped = CGI.escape(text)
  p([escaped, escaped.encoding])
  p(CGI.escapeURIComponent(text))
  p(CGI.unescape(escaped) == text)
end
long = "a b&" * 20_000
p(CGI.escape(long).bytesize)
