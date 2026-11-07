require "set"

shared = "shared"
held = Object.new
held.instance_variable_set(:@words, Set["a", "b"].freeze)
held.instance_variable_set(:@more, Set[shared])
copied = Marshal.load(Marshal.dump([held, shared, held, Set[1], shared]))
p copied[0].instance_variable_get(:@words)
p copied[0].instance_variable_get(:@more)
p copied[1], copied[2].equal?(copied[0]), copied[3], copied[4].equal?(copied[1])
p Marshal.dump(Set["a"]).bytes.size
