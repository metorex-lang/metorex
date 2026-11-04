require "json"

class Tags < Array; end
class Label < String; end
class Settings < Hash; end

GC.start
p JSON.generate(Tags["a", "b"])
GC.start
p JSON.generate(Label.new("name"))
p JSON.generate(Settings["theme" => "dark"])
extended = {"retries" => 3}
extended.extend(Comparable)
p JSON.generate(extended)

p JSON.generate({"raw" => JSON::Fragment.new("[1, 2]")})
p JSON.generate(Object.new, strict: true, as_json: ->(object, is_key) { "converted" })
coder = JSON::Coder.new { |object| object.to_s }
p coder.dump([:symbol, 1..2])

entries = Array.new(20_000) { |index| "entry#{index}" }
text = JSON.generate(entries)
p text.size
p JSON.parse(text) == entries
p JSON.parse('{"price": 9.01234567890123456789}', decimal_class: Float)["price"].class
