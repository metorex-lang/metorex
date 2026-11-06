# A constant holding an object made shareable can be read from a Ractor
# other than the main one.
Warning[:experimental] = false
require "uri"
p Ractor.shareable?(URI::RFC3986_PARSER)
p Ractor.shareable?(URI::PARSER)
p Ractor.new { URI.parse("http://example.com/path").host }.value
class Settings
  def initialize = @names = ["a", "b"]
  attr_reader :names
end
HELD = Ractor.make_shareable(Settings.new)
p Ractor.shareable?(HELD)
p Ractor.new { HELD.names }.value
