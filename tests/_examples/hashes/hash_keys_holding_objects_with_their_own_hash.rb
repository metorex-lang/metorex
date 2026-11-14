# A Hash used as a key hashes from each of its values' own #hash, so two
# Hashes holding equal values find the same entry.
class Key
  def initialize(value) = @value = value
  attr_reader :value
  def hash = value.hash
  def eql?(other) = other.is_a?(Key) && value == other.value
end

p({ x: Key.new(1) }.hash == { x: Key.new(1) }.hash)
pools = { { x: Key.new(1) } => :found }
p pools[{ x: Key.new(1) }]
p pools[{ x: Key.new(2) }]
require "uri"
cache = {}
cache[{ proxy: URI.parse("") }] ||= :first
cache[{ proxy: URI.parse("") }] ||= :second
p cache.size, cache.values
