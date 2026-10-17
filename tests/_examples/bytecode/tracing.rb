# Each TracePoint event names the line the program wrote it on.
def tally(values)
  values.each do |value|
    value * 2
  end
  values.size
end

events = []
trace = TracePoint.new(:line, :call, :return, :b_call, :class, :raise) do |point|
  events << [point.event, point.lineno]
end
trace.enable
tally([1])
class Report
end
begin
  raise ArgumentError, "bad total"
rescue ArgumentError
end
trace.disable
p events

called = []
set_trace_func proc { |event, _file, line, id, _binding, _class| called << [event, line, id] if id == :tally }
tally([2])
set_trace_func(nil)
p called.first(2)

def where
  caller_locations(1, 1).first.lineno
end
p where
