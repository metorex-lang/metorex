class StreamWrapper
  def initialize(stream)
    @stream = stream
  end

  def method_missing(name, *arguments, &block)
    @stream.__send__(name, *arguments, &block)
  end

  def respond_to_missing?(name, include_private = false)
    @stream.respond_to?(name, include_private) || super
  end
end

original = $stderr
$stderr = StreamWrapper.new($stdout)
$stderr.write("written through the wrapper\n")
$stderr = original

begin
  $stderr = Class.new.new
rescue TypeError => error
  puts error.message.sub(/0x\h+/, "0x")
end
