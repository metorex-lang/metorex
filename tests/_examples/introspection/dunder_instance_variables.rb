class Ledger
  def initialize
    @__hidden = 1
    @shown = 2
  end
end
p Ledger.new.instance_variables
class Ledger
  def initialize
    @__hidden = 1
  end
end
l = Ledger.new
p l.inspect.sub(/0x\h+/, ""), l.dup.instance_variables, l.instance_variable_get(:@__hidden)
l.instance_variable_set(:@__set, 2)
p l.instance_variables
