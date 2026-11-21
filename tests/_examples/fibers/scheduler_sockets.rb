require "socket"

class SocketLoggingScheduler
  attr_reader :events, :waited_on
  attr_accessor :addresses

  def initialize
    @events = []
    @addresses = ["127.0.0.1"]
  end

  def block(*) = Fiber.yield
  def unblock(*) = nil
  def kernel_sleep(*) = Fiber.yield
  def fiber(&block) = Fiber.new(blocking: false, &block).tap(&:resume)

  def io_wait(io, events, timeout)
    @waited_on = io
    @events << [:io_wait, io.class, events, timeout]
    Fiber.yield
  end

  def fiber_interrupt(fiber, error)
    @events << [:fiber_interrupt, error.class, error.message]
    Fiber.yield
  end

  def address_resolve(hostname)
    @events << [:address_resolve, hostname]
    @addresses
  end
end

server = TCPServer.new("127.0.0.1", 0)
port = server.connect_address.ip_port
scheduler = SocketLoggingScheduler.new
Fiber.set_scheduler(scheduler)

accepting = Fiber.new(blocking: false) { server.accept }
accepting.resume
client = TCPSocket.new("127.0.0.1", port)
IO.select([server])
accepted = accepting.resume(IO::READABLE)
client.write "ping"
p accepted.read(4)
accepted.close
client.close

refused = Fiber.new(blocking: false) { server.accept }
refused.resume
begin
  refused.resume(nil)
rescue Errno::EAGAIN => error
  p error.message
end

waiting = Fiber.new(blocking: false) { server.accept }
waiting.resume
closer = Fiber.new(blocking: false) { server.close }
closer.resume
closer.resume
begin
  waiting.resume(IO::READABLE) while waiting.alive?
rescue IOError => error
  p error.message
end
closer.resume
p [server.closed?, closer.alive?]

listener = TCPServer.new("127.0.0.1", 0)
connecting = Fiber.new(blocking: false) do
  TCPSocket.new("127.0.0.1", listener.connect_address.ip_port, connect_timeout: 2)
end
connecting.resume
IO.select(nil, [scheduler.waited_on])
connected = connecting.resume(IO::WRITABLE)
p connected.class
peer = listener.accept
peer.write "pong"
p connected.read(4)
peer.close
connected.close
listener.close

resolving = Fiber.new(blocking: false) do
  Addrinfo.getaddrinfo("example.invalid", 80, :INET, :STREAM).map(&:ip_address)
end
p resolving.resume
scheduler.addresses = []
begin
  Fiber.new(blocking: false) { Addrinfo.getaddrinfo("localhost", 80) }.resume
rescue SocketError => error
  p error.class
end
Fiber.set_scheduler(nil)
scheduler.events.each { |event| p event }

begin
  TCPSocket.new("192.0.2.1", 80, connect_timeout: 0)
rescue IO::TimeoutError => error
  p error.message
end

sockets = nil
p UNIXSocket.pair { |first, second| sockets = [first, second]; :done }
p sockets.map(&:closed?)
begin
  Socket.pair(:UNIX, :STREAM) { |first, second| sockets = [first, second]; raise "stopped" }
rescue RuntimeError => error
  p [error.message, sockets.map(&:closed?)]
end
