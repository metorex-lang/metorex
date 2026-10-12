# An autoload of a constant already defined is ignored. A file required
# while it is also autoloaded defines its constant without loading itself
# again, and `include` of an autoloaded module loads it, as RubyGems and
# Bundler do.
$VERBOSE = true

module Ledger
  Region = :north
  autoload :Region, File.join(__dir__, "autoload_lib", "ledger_region")
  autoload :Rate, File.join(__dir__, "autoload_lib", "ledger_rate")
  autoload :Rounding, File.join(__dir__, "autoload_lib", "ledger_rounding")
end

p(Ledger.autoload?(:Region))
p(Ledger::Region)
require_relative("autoload_lib/ledger_rate")
p(Ledger.autoload?(:Rate))
p(Ledger::Rate.new.rounded)
p(Ledger.autoload?(:Rounding))
autoload(:Comparable, "ledger_comparable")
p(Object.autoload?(:Comparable))
