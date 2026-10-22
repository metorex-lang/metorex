# TracePoint reports a call and a return for each operator a line calls on
# a built-in value, as it does for a named method.
events = []
var = [10, 20]
trace = TracePoint.new(:c_call, :c_return) do |tp|
  events << [tp.event, tp.method_id, tp.callee_id, tp.lineno - __LINE__] if tp.path == __FILE__
end
trace.enable
result = var[0] == 0
other = 1 + 2 < 4
trace.disable
events.each { |event| p event }
p result, other
