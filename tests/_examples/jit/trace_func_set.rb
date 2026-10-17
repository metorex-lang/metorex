events = []

def countdown(from, events)
  steps = []
  while from > 0
    if from == 2
      set_trace_func(proc { |event, _file, line| events << [event, line] if event == "line" })
    end
    steps << from
    from -= 1
  end
  set_trace_func(nil)
  steps
end

puts countdown(4, events).inspect
puts events.inspect
