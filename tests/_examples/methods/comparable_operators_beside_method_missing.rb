class Moment
  include Comparable
  attr_reader :at

  def initialize(at)
    @at = at
  end

  def <=>(other)
    at <=> other.at
  end

  def method_missing(name, *args, &block)
    at.__send__(name, *args, &block)
  end

  def respond_to_missing?(name, include_private = false)
    at.respond_to?(name, include_private)
  end

  alias_method :before?, :<
end
noon = Moment.new(12)
p [noon < Moment.new(13), noon > Moment.new(11), noon <= Moment.new(12), noon.before?(Moment.new(11))]
p noon.between?(Moment.new(1), Moment.new(20))
p noon + 1
p -noon
