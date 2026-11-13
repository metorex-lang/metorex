# A method defined inside a block in source handed to class_eval reads
# constants through the class the source was evaluated in, as Tilt's
# compiled templates do through their scope class.
module Holder
end

class Person
  CONSTANT = "Bob"
end

module Compiler
  CONSTANT = "compiler"

  def self.compile(scope)
    scope.class_eval("::Holder.class_eval do\ndef greet = CONSTANT\nend\n")
  end

  def self.read(scope)
    scope.class_eval("[CONSTANT, Module.nesting]")
  end
end

p Compiler.read(Person)

Compiler.compile(Person)
p Holder.instance_method(:greet).bind_call(Object.new)
Compiler.compile(Object)
p Holder.instance_method(:greet).bind_call(Object.new) rescue p $!.class
