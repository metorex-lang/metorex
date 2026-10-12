# `Kernel.autoload` sent to Kernel itself registers where the call sits, which
# at the top level is Object, and `Kernel.autoload?` reads it back from there.
target = File.expand_path("autoload_lib/ledger_rate.rb", __dir__)
Kernel.autoload(:LedgerRateLater, target)
p Object.const_defined?(:LedgerRateLater, false)
p Kernel.autoload?(:LedgerRateLater) == target
p Kernel.const_defined?(:LedgerRateLater, false)
