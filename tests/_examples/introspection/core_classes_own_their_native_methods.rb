require "pp"
p 5.method(:modulo).owner
p Integer.instance_method(:modulo).owner
p Numeric.instance_method(:abs2).owner
p method(:inspect).owner == singleton_class
pp self
