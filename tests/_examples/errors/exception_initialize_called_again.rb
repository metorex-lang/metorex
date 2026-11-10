# Exception#initialize called again on an exception sets its message anew,
# as parser's test helper prefixes a parse error's message with the version.
class ParseError < StandardError; end

error = ParseError.new("unexpected token")
Exception.instance_method(:initialize).bind(error).call("(33) unexpected token")
p error.message
retried = RuntimeError.new("first")
retried.send(:initialize)
p retried.message
p((retried.send(:initialize, 1, 2) rescue $!))
