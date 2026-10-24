require "strscan"
s = "ab"
scanner = StringScanner.new(s)
scanner << "cd"
p s, scanner.string.equal?(s)
begin
  StringScanner.new("ab".freeze) << "cd"
rescue => e
  p e.class
end
