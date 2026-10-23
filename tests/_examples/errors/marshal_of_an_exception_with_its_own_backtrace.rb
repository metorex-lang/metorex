class Wrapped < StandardError
  attr_accessor :inner
  def initialize(inner)
    super("wrapped")
    self.inner = inner
  end

  def backtrace
    inner.backtrace
  end
end

inner = RuntimeError.new("boom")
inner.set_backtrace(["app.rb:3"])
first = Wrapped.new(inner)
loaded = Marshal.load(Marshal.dump(first))
p Exception.instance_method(:backtrace).bind_call(loaded)
p loaded.backtrace
p loaded == first
second = Wrapped.new(inner)
second.set_backtrace(["other.rb:9"])
p first == second
