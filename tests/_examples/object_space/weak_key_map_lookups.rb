# A WeakKeyMap finds a key by its hash, asking only that key for one, and
# keeps finding each key among many.

class CountedKey
  @hash_calls = 0

  class << self
    attr_accessor :hash_calls
  end

  attr_reader :number

  def initialize(number)
    @number = number
  end

  def hash
    CountedKey.hash_calls += 1
    number.hash
  end

  def eql?(other)
    other.is_a?(CountedKey) && other.number == number
  end
end

map = ObjectSpace::WeakKeyMap.new
keys = (0...200).map { |number| CountedKey.new(number) }
keys.each { |key| map[key] = key.number * 2 }

CountedKey.hash_calls = 0
puts map[keys[150]]
puts CountedKey.hash_calls

puts keys.all? { |key| map[key] == key.number * 2 }
puts map[CountedKey.new(7)]
puts map.key?(CountedKey.new(500))
puts map.delete(keys[3])
puts map.key?(keys[3])
