# A SimpleDelegator subclass whose method_missing wraps `super` hands index
# reads and writes to the object it wraps, as thread_safe's
# SynchronizedDelegator does.
require "delegate"
require "monitor"

class Synchronized < SimpleDelegator
  def initialize(target)
    __setobj__(target)
    @monitor = Monitor.new
  end

  def method_missing(method, *args, &block)
    monitor = @monitor
    begin
      monitor.enter
      super
    ensure
      monitor.exit
    end
  end
end

hash = Hash.new { |held, key| held[key] = key.to_s }
wrapped = Synchronized.new(hash)
wrapped[1] = "one"
p wrapped[2]
p wrapped.size
p hash
list = Synchronized.new([])
list << 1
list[3] = 4
p list.__getobj__
