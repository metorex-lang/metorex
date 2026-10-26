# An identity hash finds a binary string it holds by the object, and an
# equal string that is another object is not among its keys.

held = {}.compare_by_identity
message = ("\xFF\x01" * 50_000).b
held[message] = :stored

puts held.key?(message)
puts held.key?(message.dup)
puts held[message.dup].inspect
puts held.fetch(message)
puts held.size
