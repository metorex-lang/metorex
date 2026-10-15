# A string that opens with an escaped quote, or holds only one, parses to
# the quote rather than ending early.
require "json"

p(JSON.parse('{"summary": "\"Did you mean?\" experience"}'))
p(JSON.parse('["\"", "a\\\\", "\\\\\""]'))
