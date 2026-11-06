# The match `$~` names in each turn of a gsub or scan block, while other
# matches are made in other text between them, and after the string matched
# is changed.
text = +"one=1, two=2, three=3"
seen = []
replaced = text.gsub(/(\w+)=(\d)/) do
  match = $~
  "other text 9".match(/\d/)
  seen << [match.pre_match.size, match[1], match.begin(2), match.string.frozen?, match.string.equal?(text)]
  "#{$2}=#{$1}"
end
p(replaced)
seen.each { |entry| p(entry) }

strings = []
text.scan(/\d/) { strings << $~.string }
p(strings.uniq.size)
p(strings.first == text)
text << " four=4"
p(strings.first)
p($~[0])

wide = "été = summer, éclair = pastry"
wide.scan(/(\w+) = (\w+)/) { p([$~.begin(1), $~.end(2), $1, $2]) }
