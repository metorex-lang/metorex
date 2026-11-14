# The NameError Kernel#method raises names the receiver's singleton class
# when it has one, as every class does, and its class otherwise.
module WithOwnMethod
  def self.ready; end
end

class Plain; end

with_singleton = Object.new
def with_singleton.ready; end

[Plain, "text", Comparable, WithOwnMethod, Math, with_singleton].each do |receiver|
  receiver.method(:missing)
rescue NameError => error
  puts error.message.sub(/0x\h+/, "0x")
end
