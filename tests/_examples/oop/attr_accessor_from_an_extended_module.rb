module Recorded
  def self.included(klass)
    klass.extend ClassMethods
  end

  module ClassMethods
    def recorded_names
      @recorded_names ||= []
    end

    def attr_accessor(*names)
      recorded_names.concat(names)
      super
    end
  end
end

class Shipment
  include Recorded

  attr_accessor :status
  attr_accessor :carrier
end

p Shipment.recorded_names
shipment = Shipment.new
shipment.status = :sent
p shipment.status
