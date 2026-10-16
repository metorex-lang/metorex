# Definitions written where a body compiles them: at the top level and in
# methods. Class and module bodies run on the tree walker for now.

def greeting(name = "you", *rest, key: 1, **others, &block)
  [name, rest, key, others, block&.call]
end

module Shout
  def shout
    "#{self}!"
  end
end

class Plain
end

module Outer
end

module Outer::Inner
end

class Outer::Kind < Struct.new(:size)
end

class String
  include Shout
end

alias hello greeting

held = Object.new
class << held
  def speak = :spoken
end

class << (twin = Object.new)
  def speak = :twin
end

def define_later
  def made_later = :later
end

define_later
p greeting
p hello("ann", 1, key: 2, extra: 3) { :block }
p "hi".shout
p [Plain.name, Outer::Inner.name, Outer::Kind.new(3).size, Outer::Kind.superclass.superclass]
p [held.speak, twin.speak, made_later]
