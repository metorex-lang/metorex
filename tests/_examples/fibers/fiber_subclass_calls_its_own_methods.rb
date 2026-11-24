class ThreadlikeFiber < Fiber
  def join(timeout = nil)
    now = Time.now
    resume while alive? && (!timeout || Time.now - now < timeout)
  end

  def kill_quietly
    kill if respond_to?(:kill)
  end
end

steps = []
fiber = ThreadlikeFiber.new do
  steps << 1
  Fiber.yield
  steps << 2
end
fiber.join
p steps
p fiber.alive?

class TimeoutError < StandardError
end

raising = ThreadlikeFiber.new { raise TimeoutError, "no connection" }
begin
  raising.join
rescue TimeoutError => error
  p [error.class, error.message]
end
p raising.alive?
