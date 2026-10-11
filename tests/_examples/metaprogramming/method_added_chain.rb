module Tracking
  def method_added(name)
    super(name)
    (@seen ||= []) << name
  end

  def singleton_method_added(name)
    super(name)
    (@singletons ||= []) << name
  end
end

module Counting
  def method_added(name)
    super
    @count = (@count || 0) + 1
  end
end

class Rates
  extend(Tracking)
  extend(Counting)

  class << self
    def method_added(name)
      super
      @last = name
    end

    attr_reader(:seen, :count, :last, :singletons)
  end

  attr_writer :region
  attr_accessor :currency
  def load_rates = 1
  def store_rates = 2
  def self.refresh = 3
end

p([Rates.seen, Rates.count, Rates.last])
p(Rates.singletons)
