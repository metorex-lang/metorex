require "strscan"
pattern = +"ab"
scanner = StringScanner.new("ababcd")
p scanner.scan(pattern), scanner.scan(pattern)
pattern.replace("cd")
p scanner.scan(pattern)
scanner = StringScanner.new("a.b" * 40)
p (1..40).map { |step| scanner.skip("a.b") }.uniq, scanner.eos?
p StringScanner.new("ab").scan("a.")
words = StringScanner.new((0...40).map { |step| "w#{step};" }.join)
p (0...40).all? { |step| words.skip("w#{step};") }, words.eos?
