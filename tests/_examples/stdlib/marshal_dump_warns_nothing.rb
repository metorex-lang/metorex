# Marshal.dump of a Struct, or of a copy of one, leaves the literal strings
# it holds as they were, so no warning is printed when warnings are on.
$VERBOSE = true
Pair = Struct.new(:left, :right)
pair = Pair.new(7, "something")
p Marshal.load(Marshal.dump(pair))
p Marshal.dump(pair.dup) == Marshal.dump(pair)
p pair.right.frozen?
Warning[:deprecated] = true
label = "something"
Marshal.dump(label)
Marshal.dump([label, { label => label }])
module Tagged; end
tagged = "tagged".dup.extend(Tagged)
p Marshal.load(Marshal.dump(tagged)).singleton_class.include?(Tagged)
