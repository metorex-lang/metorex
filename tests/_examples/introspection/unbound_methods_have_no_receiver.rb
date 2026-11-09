module Greeting
  def hello = :hello
end

unbound = Greeting.instance_method(:hello)
p unbound.respond_to?(:receiver)
begin
  unbound.receiver
rescue NoMethodError => error
  p error.class
end

bound = nil.method(:to_s)
p bound.respond_to?(:receiver)
p bound.receiver
