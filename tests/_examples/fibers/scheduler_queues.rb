class QueueLoggingScheduler
  attr_reader :events

  def initialize
    @events = []
  end

  def block(blocker, timeout)
    @events << [:block, blocker.class, timeout]
    Fiber.yield
  end

  def unblock(blocker, fiber)
    @events << [:unblock, blocker.class]
  end

  def kernel_sleep(*) = Fiber.yield
  def io_wait(*) = Fiber.yield
  def fiber_interrupt(*) = nil

  def fiber(&block)
    Fiber.new(blocking: false, &block).tap(&:resume)
  end
end

scheduler = QueueLoggingScheduler.new
Fiber.set_scheduler(scheduler)

queue = Queue.new
popped = nil
popper = Fiber.schedule { popped = queue.pop }
p queue.num_waiting
queue << 42
popper.resume while popper.alive?
p popped

sized = SizedQueue.new(1)
sized << 1
pusher = Fiber.schedule { sized << 2 }
p sized.pop
pusher.resume while pusher.alive?
p sized.pop

closed_error = nil
sized << 1
blocked = Fiber.schedule do
  sized << 2
rescue ClosedQueueError => error
  closed_error = error.class
end
sized.close
blocked.resume while blocked.alive?
p closed_error

thread_queue = Queue.new
thread = Thread.new { thread_queue.pop }
Thread.pass until thread.stop?
joiner = Fiber.schedule { thread.join(10) }
thread_queue << :done
Thread.pass until scheduler.events.size == 8
p joiner.resume.equal?(thread)

Fiber.set_scheduler(nil)
scheduler.events.each { |event| p event }
