x = /.+/
expected = %r{\A/(#{x})(?:/([^/.?]+))?\Z}
body = "/" + "(#{Regexp.union(/.+/)})" + "(?:" + "/" + "([^/.?]+)" + ")?"
built = %r{\A#{body}\Z}
p expected.source
p built.source
p [expected.options, built.options]
p [expected.encoding, built.encoding]
p [expected.fixed_encoding?, built.fixed_encoding?]
p expected == built
p expected.hash == built.hash
p %r{a/b}.to_s, Regexp.new("a/b").to_s, %r{a\/b}.inspect
p %r{a/b} == /a\/b/, %r{a\/b} == /a\/b/, %r{[/]}.source, /a\/#{x}/.source
