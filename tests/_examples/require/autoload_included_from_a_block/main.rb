# A block names a constant through the modules around where it was written,
# so `include Default` in a block a class runs with class_eval loads the
# autoload Ledger::Scoping registered, as Active Record's `included do`
# blocks do.
module Ledger
  module Scoping
    autoload :Default, File.join(__dir__, "ledger_default")

    def self.included_block
      proc { include Default }
    end
  end
end

class Account; end
Account.class_eval(&Ledger::Scoping.included_block)
p Account.include?(Ledger::Scoping::Default)

# A module's own autoload comes before a constant of the same name further
# out, so `include QueryCache` in Ledger::Adapters loads the adapters' module
# though Ledger::QueryCache is a class already loaded.
module Ledger
  class QueryCache; end

  module Adapters
    autoload :QueryCache, File.join(__dir__, "query_cache")

    class SqliteAdapter
      include QueryCache
    end
  end
end
p Ledger::Adapters::SqliteAdapter.new.cached?
