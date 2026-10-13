module Definitions
  def calculator = :from_module
end

class Group
  include Definitions
  define_method(:calculator) { [:group, super(&nil)] }
end

class Nested < Group
  include Module.new
end

class Plain < Group
end

p Plain.new.calculator
p Nested.new.calculator
p Group.instance_method(:calculator).owner
