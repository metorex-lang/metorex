module Ledger
  module Scoping
    module Default
      def self.included(base)
        puts "Ledger::Scoping::Default included in #{base}"
      end
    end
  end
end
