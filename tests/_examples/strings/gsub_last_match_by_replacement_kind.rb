text = "a1 b2 c3"

p text.gsub(/(\w)(\d)/, '\2\1')
p $~[0], $~.pre_match

p text.gsub(/(\w)(\d)/) { "#{$~[2]}#{$1}" }
p $~[0]

seen = []
table = Hash.new { |_, key| seen << $~[0]; key.upcase }
p text.gsub(/[a-c]/, table)
p seen, $~[0]

p text.gsub(/x/, "y"), $~

long = "aaaa \t" * 50_000
p long.gsub(/\s/, " ").squeeze(" ").length
