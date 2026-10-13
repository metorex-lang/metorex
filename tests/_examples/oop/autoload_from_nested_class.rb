$LOAD_PATH.unshift(File.join(__dir__, "autoload_fixture"))

module Shipping
  autoload :Rates, "shipping_rates"

  class Quote
    def total = Rates.flat + 3
  end
end

class Order
end

Order.module_exec do
  p(Shipping::Quote.new.total)
end
