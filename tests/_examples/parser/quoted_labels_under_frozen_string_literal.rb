# frozen_string_literal: true

# A quoted label names a Symbol under the frozen_string_literal comment, in
# a Hash literal and as a keyword argument.
headers = {
  'parts': { 'text': "plain", "file": "binary" },
  "id": 1
}
p(headers)
p(headers.keys.map(&:class))

def keywords(**given)
  given
end

p(keywords('single': 1, "double": 2))
p(keywords(**{ 'spread': 3 }))

name = "dyn"
p({ "#{name}amic": 1 })
p(keywords("#{name}amic": 2))
