# The Kernel methods every object answers can be taken as Method objects in
# a module body, as the debug gem takes `singleton_class` and `class`.
module Ledger
  SINGLETON_CLASS = method(:singleton_class).unbind
  CLASS = method(:class).unbind
end

p Ledger::SINGLETON_CLASS
p Ledger::SINGLETON_CLASS.bind_call Ledger
p Ledger::CLASS.bind_call "rate"
p Object.new.method(:singleton_class).owner
p Ledger.method(:singleton_class).call
