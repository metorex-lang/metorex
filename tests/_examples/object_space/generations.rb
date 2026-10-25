# Cycles freed by the collections that run on their own, and a cycle that
# survived those collections freed once GC.start covers every object.

class Peer
  attr_accessor :peer
end

class KeptPeer
  attr_accessor :peer
end

def make_pair(kind)
  first = kind.new
  second = kind.new
  first.peer = second
  second.peer = first
  first
end

kept = make_pair(KeptPeer)
20_000.times { make_pair(Peer) }
puts ObjectSpace.each_object(Peer).count < 20_000
puts ObjectSpace.each_object(KeptPeer).count
kept = nil
GC.start
puts ObjectSpace.each_object(KeptPeer).count
GC.start
puts ObjectSpace.each_object(Peer).count
