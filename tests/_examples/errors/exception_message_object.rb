class Reason
  def to_s = "disk is full"
end

class Unprintable
  def to_s = raise(ArgumentError, "cannot print")
end

error = RuntimeError.new(Reason.new)
puts error.message
puts error.to_s
p error.message.class

broken = Exception.new(Unprintable.new)
begin
  broken.message
rescue ArgumentError => raised
  puts raised.message
end
p StandardError.new(42).message
