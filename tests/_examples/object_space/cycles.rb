# Objects that refer to each other, freed by the cycle collector.

class Peer
  attr_accessor :peer
end

class Holder
  def initialize
    @callback = -> { self }
  end
end

# A finalizer made here holds none of the objects it is defined for.
def finalizer(label)
  proc { puts "#{label} finalized" }
end

def make_pair
  first = Peer.new
  second = Peer.new
  first.peer = second
  second.peer = first
  ObjectSpace.define_finalizer(first, finalizer("pair"))
  nil
end

def make_holder
  holder = Holder.new
  ObjectSpace.define_finalizer(holder, finalizer("holder"))
  nil
end

# A collection here leaves what startup made out of the count toward the
# next automatic one, so none runs before the GC.start below.
GC.start
make_pair
make_holder
puts "before"
runs = GC.count
GC.start
puts "after"
puts GC.count == runs + 1

# A WeakMap entry goes once its key is freed, and stays while it is held.
def add_entry(map, value)
  key = Object.new
  map[key] = value
  map.size
end

map = ObjectSpace::WeakMap.new
value = "kept value"
held_key = Object.new
map[held_key] = value
puts add_entry(map, value)
GC.start
puts map.size
puts map[held_key]

# each_object finds a String.
marker = "each_object marker"
puts ObjectSpace.each_object(String).any? { |one| one.equal?(marker) }

# Cycles made in a loop are freed as the list of objects grows.
20_000.times do
  looped = Peer.new
  looped.peer = looped
end
puts ObjectSpace.each_object(Peer).count < 20_000

# GC.stat counts the objects freed.
puts GC.stat(:total_freed_objects) > 0
