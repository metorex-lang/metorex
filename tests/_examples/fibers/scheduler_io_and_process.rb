require "tmpdir"

class IoLoggingScheduler
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
    events
  end

  def io_read(io, buffer, length, offset)
    @events << [:io_read, buffer.size, length, offset]
    Fiber.blocking { buffer.read(io, length, offset) }
  end

  def process_wait(pid, flags)
    @events << [:process_wait, pid == -1 ? -1 : :child, flags]
    Fiber.blocking { Process::Status.wait(pid, flags) }
  end
end

class NilStatusScheduler < IoLoggingScheduler
  def process_wait(pid, flags)
    Fiber.blocking { Process::Status.wait(pid, flags) }
    nil
  end
end

Dir.mktmpdir do |directory|
  path = File.join(directory, "greeting.txt")
  File.write(path, "Hello World!")

  scheduler = IoLoggingScheduler.new
  Fiber.set_scheduler(scheduler)
  results = nil
  Fiber.schedule do
    reader, writer = IO.pipe
    results = [
      File.read(path),
      File.open(path) { |file| [file.read(5), file.read] },
      writer.wait_writable(1).equal?(writer),
      reader.wait_readable.equal?(reader),
    ]
    reader.close
    writer.close
    Process.spawn("sh", "-c", "exit 3")
    waited = Process.waitall
    results << [waited.size, waited[0][1].exitstatus]
  end
  Fiber.set_scheduler(nil)
  results.each { |result| p result }
  scheduler.events.each { |event| p event }
end

Fiber.set_scheduler(NilStatusScheduler.new)
Fiber.schedule do
  system("true")
rescue TypeError => error
  p error.message
end
Fiber.set_scheduler(nil)

reader, writer = IO.pipe
writer.write "abcdef"
writer.close
buffer = IO::Buffer.new(4)
p buffer.read(reader, 2, 1)
p buffer.get_string(1)
p buffer.read(reader)
p buffer.read(reader)

whole = IO::Buffer.new(8)
begin
  whole.slice(0, 4).xor!(whole.slice(2, 4))
rescue IO::Buffer::MaskError => error
  p error.message
end
p whole.slice(0, 4).xor!(whole.slice(4, 4)).size

length = Object.new
def length.to_int = "four"
begin
  File.open(__FILE__) { |file| file.read(length) }
rescue TypeError => error
  p error.message.sub(/#<Object:0x\h+>/, "Object")
end
