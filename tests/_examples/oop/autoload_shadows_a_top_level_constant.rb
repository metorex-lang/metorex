$LOAD_PATH.unshift(File.join(__dir__, "autoload_fixture"))

class Table
  def self.source = :top_level
end

module Shipping
  autoload :Table, "shipping_rates_table"

  class Quote
    def source = Table.source
  end
end

p(Shipping::Quote.new.source)
p(Table.source)
