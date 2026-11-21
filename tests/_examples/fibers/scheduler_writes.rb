require "io/nonblock"

class WriteLoggingScheduler
  attr_reader :events

  def initialize
    @events = []
  end

  def block(*) = Fiber.yield
  def unblock(*) = nil
  def kernel_sleep(*) = Fiber.yield
  def fiber_interrupt(*) = nil
  def fiber(&block) = Fiber.new(blocking: false, &block).tap(&:resume)

  def io_wait(io, events, timeout)
    @events << [:io_wait, io.class, events, timeout]
    Fiber.yield
  end
end

def drain(reader)
  taken = 0
  loop do
    chunk = reader.read_nonblock(65536, exception: false)
    return taken if chunk == :wait_readable
    taken += chunk.bytesize
  end
end

reader, writer = IO.pipe
writer.nonblock = true
filled = 0
until (written = writer.write_nonblock("x" * 65536, exception: false)) == :wait_writable
  filled += written
end

scheduler = WriteLoggingScheduler.new
Fiber.set_scheduler(scheduler)
writers = [Fiber.schedule { writer.write("ab") }, Fiber.schedule { writer.syswrite("cd") }]
p writers.map(&:alive?)
p drain(reader) == filled
results = writers.map do |fiber|
  result = nil
  result = fiber.resume(IO::WRITABLE) while fiber.alive?
  result
end
p results
p reader.read_nonblock(4)
Fiber.set_scheduler(nil)
scheduler.events.each { |event| p event }
reader.close
writer.close
