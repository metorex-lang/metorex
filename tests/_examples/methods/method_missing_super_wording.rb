class Forwarder
  def method_missing(name, *arguments) = super
end

begin
  Forwarder.new.shipped(1)
rescue NoMethodError => error
  puts error.message
end

forwarder = Forwarder.new
def forwarder.tracked = true

begin
  forwarder.shipped(1)
rescue NoMethodError => error
  puts error.message.sub(/0x\h+/, "0x")
end
