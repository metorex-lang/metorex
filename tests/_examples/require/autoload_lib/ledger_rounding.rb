module Ledger
  module Rounding
    def rounded = (amount * 100).round / 100.0
  end
end
