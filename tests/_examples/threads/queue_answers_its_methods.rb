queue = Thread::Queue.new
p queue.respond_to?(:empty?)
p queue.respond_to?(:pop)
p queue.respond_to?(:num_waiting)
p queue.empty?
sized = Thread::SizedQueue.new(2)
p sized.respond_to?(:empty?)
p sized.respond_to?(:max)
