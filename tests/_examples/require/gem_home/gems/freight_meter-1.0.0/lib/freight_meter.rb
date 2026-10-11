require "freight_meter/meter"
require "tariff_table"

module FreightMeter
  def self.with_base(weights)
    charges(weights, TariffTable::BASE)
  end
end
