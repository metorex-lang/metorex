identity = {}.compare_by_identity
identity["foo".dup] = :bar
identity["foo".dup] = :glark
copied = identity.dup
cloned = identity.clone
puts copied == identity
puts cloned == identity
puts copied.size
puts copied.compare_by_identity?

kept = {}.compare_by_identity
foo = "foo"
kept[foo] = true
kept[foo] = true
puts kept.size
puts kept.keys.first.equal?(foo)
