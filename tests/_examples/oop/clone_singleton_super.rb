module Counting
  def total(values) = values.sum
end

ledger = Object.new
ledger.singleton_class.include(Counting)
ledger.singleton_class.define_method(:total) { |values| super(values) + 1 }
copy = ledger.clone
p copy.total([1, 2])
p copy.method(:total).owner == copy.singleton_class

module Reporting
  def heading = "report"
end
copied_module = Reporting.dup
p copied_module.instance_method(:heading).owner == copied_module
