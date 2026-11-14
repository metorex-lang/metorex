# A producer waiting on a full SizedQueue gets ClosedQueueError when the
# queue closes, even when room is made on it after it closed.
q = SizedQueue.new(1)
release = Queue.new
started = Queue.new
worker = Thread.new do
  while (job = q.shift)
    job.call
  end
end
q.push(-> { started << :running; release.pop })
started.pop
q.push(-> {})
errors = []
producer = Thread.new do
  q.push(-> {})
rescue ClosedQueueError => error
  errors << error
end
sleep 0.1
p producer.status
q.close
worker.kill
q.clear
p producer.join(2).nil?
p errors.map(&:class)
