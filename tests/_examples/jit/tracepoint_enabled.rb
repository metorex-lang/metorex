lines = []
trace = TracePoint.new(:line) { |point| lines << point.lineno }

def countdown(from, trace)
  steps = []
  while from > 0
    trace.enable if from == 2
    steps << from
    from -= 1
  end
  trace.disable
  steps
end

puts countdown(4, trace).inspect
puts lines.inspect
