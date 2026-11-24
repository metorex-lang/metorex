class Forwarder < BasicObject
  def initialize(target)
    @target = target
  end

  def method_missing(name, *args, **kwargs, &block)
    @target.send(name, *args, **kwargs, &block)
  end
end

def check(held, klass)
  held.kind_of?(klass)
end

p check("text", String)
p check(Forwarder.new("text"), String)
p check(Forwarder.new("text"), String)
p check(1, Integer)
p check(Forwarder.new(1), Integer)
class SendingChecker
  define_method(:send, ::Kernel.instance_method(:send))

  def check
    [Forwarder.new("text").kind_of?(String), Forwarder.new(1).frozen?]
  end
end

p SendingChecker.new.check
p SendingChecker.new.send(:check)
