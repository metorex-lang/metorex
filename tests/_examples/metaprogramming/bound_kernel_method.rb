class Ledger
  def record(entry, amount = 0) = [entry, amount]
  def method(*) = "a method of its own"
end

ledger = Ledger.new
bound = Kernel.instance_method(:method).bind(ledger)
recorded = bound.call(:record)
p recorded.class
p recorded.arity
p recorded.call("rent", 5)
p ledger.method(:record)
p Kernel.instance_method(:method).bind_call(ledger, :record).name
