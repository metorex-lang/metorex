module Ledger
  module Adapters
    class NullTransaction; end
    class TransactionManager
      NULL = NullTransaction.new
    end
  end
end
