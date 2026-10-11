require "tariff_table"
require "shipping_rates/zones"

module ShippingRates
  VERSION = "1.2.0"

  def self.for(kilograms) = kilograms * TariffTable::BASE
end
