default = $/
p default.frozen?
handed = "\n".dup.freeze
$/ = handed
p [$/.equal?(handed), $/.equal?(default), $/.frozen?]
$-0 = "\n".dup
p [$/.equal?(default), $-0.equal?(default)]
other = "abc".freeze
$/ = other
p $/.equal?(other)
$/ = "\n"
p $/.equal?(default)
