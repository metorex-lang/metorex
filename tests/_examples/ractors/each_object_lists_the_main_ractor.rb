$VERBOSE = nil
before = ObjectSpace.each_object(Ractor).to_a
p before.size
worker = Ractor.new { receive * 2 }
worker.send 21
p worker.value
p (ObjectSpace.each_object(Ractor).to_a - before).map { |held| held.inspect.include?("terminated") }
