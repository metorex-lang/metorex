order = []
first = TracePoint.new(:class) { order << :first }
second = TracePoint.new(:class) { order << :second }
first.enable do
  second.enable do
    class NewestTracepointRunsFirst
    end
  end
end
p order
