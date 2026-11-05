# CGI.escape writes each byte of what it is given, and answers in that
# text's encoding.
require "cgi"

latin = CGI.escape("é".encode(Encoding::ISO_8859_1))
p latin, latin.encoding
binary = CGI.escape("\x80".b)
p binary, binary.encoding
p CGI.escape("a b~é"), CGI.escape("a b~é").encoding
