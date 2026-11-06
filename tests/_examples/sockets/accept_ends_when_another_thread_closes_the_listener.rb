# A thread waiting in accept is woken with an IOError when another thread
# closes the listener.
require "socket"
server = TCPServer.new("127.0.0.1", 0)
waiter = Thread.new do
  server.accept
rescue IOError => error
  [error.class, error.message]
end
Thread.pass until waiter.status == "sleep"
server.close
p waiter.value
