# power_assert's load check: a TracePoint on :return and :c_return sees the
# call of `__id__` in the block `enable` runs, with its return value and the
# name it was called by. A method Ruby writes in C fires `c_return` called
# with a receiver or without.
seen = []
TracePoint.new(:return, :c_return) do |trace|
  seen << [trace.event, trace.callee_id, trace.return_value.is_a?(Integer)]
end.enable { __id__ }
p seen

ledger = Object.new
seen = []
TracePoint.new(:c_return) { |trace| seen << trace.method_id }.enable do
  ledger.__id__
  __id__
  self.__id__
  "total".upcase
end
p seen
