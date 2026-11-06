# open-uri's Meta reads the charset a content-type names and gives the
# String it was added to that encoding.
require "open-uri"
held = "".dup
OpenURI::Meta.init(held)
held.meta_add_field("content-type", "text/plain; charset=ISO-8859-1")
p held.encoding
p held.content_type
p held.charset
