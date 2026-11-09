module Anything
  def self.===(_) = true
end
queue = Queue.new
reader = Thread.new do
  begin
    queue.pop
  rescue Anything => error
    puts "rescued #{error.message}"
  end
  puts "kept going"
end
Thread.pass until reader.status == "sleep"
reader.kill
p reader.raise(Interrupt)
reader.join
p reader.status
