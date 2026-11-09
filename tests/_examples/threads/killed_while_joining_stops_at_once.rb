state = [nil, nil]
children = [nil, nil]
outer = Thread.new do
  parent = Thread.current
  threads = [0, 1].map do |i|
    Thread.new do
      children[i] = Thread.current
      state[i] = :ready
      parent.join
      state[i] = :error
    end
  end
  threads.each(&:join)
ensure
  threads&.each(&:kill)
end
Thread.pass while state.any?(&:nil?)
outer.kill
Thread.pass while children.any?(&:alive?)
p state
