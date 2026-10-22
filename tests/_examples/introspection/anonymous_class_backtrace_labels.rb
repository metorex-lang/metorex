anonymous = Class.new do
  def failing
    raise "stop"
  end

  def self.failing_on_the_class
    raise "stop"
  end
end

begin
  anonymous.new.failing
rescue => error
  puts error.backtrace_locations.first.label
  puts error.backtrace.first[/'.*'/]
end

begin
  anonymous.failing_on_the_class
rescue => error
  puts error.backtrace_locations.first.label
end

anonymous_module = Module.new do
  def self.failing
    raise "stop"
  end
end

begin
  anonymous_module.failing
rescue => error
  puts error.backtrace_locations.first.label
end
