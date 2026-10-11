class Walker
  def each_item &block
    [1, 2].each(&block)
  end

  def each_pair_of &block
    { a: 1 }.each_pair(&block)
  end
end

module Shipping
  RATE = 4

  class Entry
    def initialize value
      @value = value
    end

    def to_s = "entry #{@value}"
  end

  def self.collect
    found = []
    Walker.new.each_item { |item| found << Entry.new(item * RATE).to_s }
    Walker.new.each_pair_of { |key, value| found << "#{key} #{value + RATE}" }
    found
  end
end

p Shipping.collect
