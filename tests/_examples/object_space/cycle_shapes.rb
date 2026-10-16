# Each shape of cycle GC.start frees.

class Peer
  attr_accessor :peer
end

class Linked
  def link(other)
    @peer = other
  end
end

def finalizer(label)
  proc { puts "#{label} finalized" }
end

# The first finalizer a program defines is for an object in a cycle.
def first_finalized
  one = Peer.new
  one.peer = one
  ObjectSpace.define_finalizer(one, finalizer("first"))
  nil
end

def pair_by_writer
  first = Peer.new
  second = Peer.new
  first.peer = second
  second.peer = first
  nil
end

def pair_by_method
  first = Linked.new
  second = Linked.new
  first.link(second)
  second.link(first)
  nil
end

def array_cycle
  list = [:array_cycle]
  list << list
  nil
end

def hash_cycle
  table = { name: :hash_cycle }
  table[:itself] = table
  nil
end

def block_cycle
  held = Peer.new
  held.peer = proc { held }
  nil
end

first_finalized
pair_by_writer
pair_by_method
array_cycle
hash_cycle
block_cycle
GC.start
p ObjectSpace.each_object(Peer).count
p ObjectSpace.each_object(Linked).count
p ObjectSpace.each_object(Array).count { |one| one.first == :array_cycle }
p ObjectSpace.each_object(Hash).count { |one| one.first == [:name, :hash_cycle] }
