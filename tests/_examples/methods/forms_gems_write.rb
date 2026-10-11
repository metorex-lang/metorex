# Forms installed gems write: an exception's `initialize` reaching its
# parent's through `super`, `attr_reader(*names)` from a method, `include`
# and `extend` under a modifier, a module's `new` extended onto a parent
# class, `super` handed a block or a splat without parentheses, and a
# modifier on the first line of a `case` clause used as a value.
class LookupError < StandardError
  attr_reader :key

  def initialize(key, detail)
    @key = key
    super("#{key}: #{detail}")
  end
end

class MissingKeyError < LookupError
  def initialize(key)
    super(key, "not found")
  end
end

error = MissingKeyError.new("sku")
p([error.key, error.message])

module Readers
  def private_reader(*names)
    attr_reader(*names)
    private(*names)
  end
end

class Parcel
  extend Readers
  private_reader :weight, :zone

  def initialize
    @weight = 4
    @zone = "north"
  end

  def label = "#{weight}kg to #{zone}"
end
p(Parcel.new.label)

module Tracked
  def new(*arguments, &block)
    super(*arguments, &block)
  ensure
    $made = ($made || 0) + 1
  end
end

module Labelled
end

class Shipment
  include Labelled if true

  def self.track! = (extend Tracked unless singleton_class < Tracked)
end

class ExpressShipment < Shipment
  track!
end

class OvernightShipment < ExpressShipment
  track!
end
p([OvernightShipment.new.class, ExpressShipment.new.class, $made])
p(Shipment.include?(Labelled))

class Courier
  def deliver(*stops, **options)
    [stops, options, (yield if block_given?)]
  end
end

class LocalCourier < Courier
  def deliver(first, second, &note)
    super first, second, &note
  end
end

class NightCourier < Courier
  def deliver(*stops)
    super *stops, late: true
  end
end
p(LocalCourier.new.deliver(1, 2) { :signed })
p(NightCourier.new.deliver(3))

def rate_for(zone)
  case zone
  when :north
    5
  else
    raise ArgumentError, "unknown zone" unless zone
    9
  end
end
p([rate_for(:north), rate_for(:south)])
