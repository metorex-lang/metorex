class UnblockScheduler
  attr_reader :unblocked

  def initialize
    @unblocked = Thread::Queue.new
  end

  def block(blocker, timeout = nil)
    Fiber.yield
  end

  def unblock(blocker, fiber)
    @unblocked << fiber
  end

  def kernel_sleep(duration = nil)
    Fiber.yield
  end

  def io_wait(io, events, timeout)
    Fiber.yield
  end

  def fiber_interrupt(fiber, exception)
    fiber.raise(exception)
  end

  def fiber(&block)
    Fiber.new(blocking: false, &block).tap(&:resume)
  end
end

scheduler = UnblockScheduler.new
mutex = Mutex.new
locked = Thread::Queue.new
finish = Thread::Queue.new
holder = Thread.new do
  mutex.lock
  locked << true
  finish.pop
end
locked.pop

Fiber.set_scheduler(scheduler)
waiter = Fiber.new(blocking: false) do
  mutex.lock
  :locked
end
waiter.resume

finish << true
holder.join

p scheduler.unblocked.size
p scheduler.unblocked.pop.equal?(waiter)
p waiter.resume
Fiber.set_scheduler(nil)
