require "delegate"

empty = SimpleDelegator.allocate
p empty.respond_to?(:succ)
p empty.respond_to?(:__getobj__)
p empty.__getobj__ { :nothing_delegated }
begin
  empty.__getobj__
rescue ArgumentError => error
  p error.message
end
empty.__setobj__(41)
p empty.respond_to?(:succ), empty.succ
