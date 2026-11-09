def report
  ledger = Object.new
  def ledger.total = 42
  held_total = ledger.method(:total)
  unbound = Integer.instance_method(:to_s)
  [binding.local_variables, eval("held_total").call, eval("unbound").name]
end
p report
def helper; end
p local_variables
held = 1.method(:+)
p defined?(held), defined?(helper)
