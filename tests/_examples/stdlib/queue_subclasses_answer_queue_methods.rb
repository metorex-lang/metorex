# A subclass of Queue or SizedQueue makes a queue and answers its methods,
# as concurrent-ruby's TimeoutQueue subclasses Queue.
class TimeoutQueue < ::Queue; end
queue = TimeoutQueue.new
queue << 1
queue.push(2)
p queue.pop, queue.pop(timeout: 0), queue.pop(timeout: 0), queue.size
p TimeoutQueue.new([3, 4]).pop
class Bounded < Thread::SizedQueue; end
bounded = Bounded.new(2)
bounded << :a
p bounded.max, bounded.pop
class Labeled < ::Queue
  attr_reader :label
  def initialize(label)
    @label = label
    super()
  end
end
labeled = Labeled.new("jobs")
labeled << :job
p labeled.label, labeled.pop, labeled.empty?
