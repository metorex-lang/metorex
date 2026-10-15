# StringScanner reads groups by number and by name, and counts what it
# matched in bytes.
require "strscan"

scanner = StringScanner.new("ab12")
scanner.scan(/(?<l>[a-z]+)/)
p([scanner[:l], scanner["l"], scanner[1], scanner[-1]])
p([scanner.captures, scanner.named_captures, scanner.size, scanner.values_at(0, 1)])
begin
  scanner[:missing]
rescue IndexError => error
  p(error.message)
end
scanner.scan(/(\d)(x)?/)
p([scanner.captures, scanner[2], scanner[5], scanner.named_captures])
p([scanner.scan(/(?<a>.)|(?<a>z)/), scanner.named_captures])

text = StringScanner.new("héllo wörld\nnext")
p([text.scan(/h./), text.pos, text.scan_until(/w/), text.pre_match, text.post_match])
p([text.check(/ö/), text.skip(/\S+/), text.bol?, text.scan(/\s/), text.bol?, text.scan(/^n/), text.rest])

bytes = StringScanner.new("abc".b + "\xff\xfe".b)
p([bytes.scan(/abc/n), bytes.scan(/\xff/n), bytes.pos, bytes.rest])

counted = StringScanner.new("éé x")
p([counted.match?(/é/), counted.skip(/é/), counted.matched_size, counted.skip_until(/x/), counted.exist?(/z/)])
counted.reset
p([counted.search_full(/x/, false, false), counted.scan_full(/é/, false, false), counted.check_until(/x/), counted.exist?(/x/)])
