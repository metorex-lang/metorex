require "monitor"

monitor = Monitor.new
arrived = monitor.new_cond
replies = []

reader = Thread.new do
  3.times do |number|
    monitor.synchronize do
      replies << "reply #{number}"
      arrived.signal
    end
    Thread.pass
  end
end

3.times do |number|
  monitor.synchronize do
    arrived.wait_until { replies.size > number }
    puts replies[number]
  end
end
reader.join

class Mailbox
  include MonitorMixin

  def initialize
    super
    @letters = []
    @delivered = new_cond
  end

  def deliver(letter)
    synchronize do
      @letters << letter
      @delivered.broadcast
    end
  end

  def take
    synchronize do
      @delivered.wait_while { @letters.empty? }
      @letters.shift
    end
  end
end

mailbox = Mailbox.new
taker = Thread.new { mailbox.take }
Thread.pass
mailbox.deliver("invoice 1042")
puts taker.value

held = Monitor.new
held.enter
other = Thread.new { held.try_enter }
puts other.value
held.exit

begin
  held.exit
rescue ThreadError => error
  puts error.message
end

puts held.new_cond.wait(0.01) rescue puts $!.message
