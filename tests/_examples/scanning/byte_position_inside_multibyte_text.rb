require "strscan"
scanner = StringScanner.new("aé日本x")
[0, 1, 3, 4, 6, 9, 10].each do |offset|
  scanner.pos = offset
  p [offset, scanner.pos, scanner.charpos, scanner.rest]
end
