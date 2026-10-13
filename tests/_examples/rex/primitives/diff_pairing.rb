# Rex's differ pairs the lines of two texts through Rex::Diff.__matches__,
# which answers, for each line of the first, the line of the second it is
# paired with, or nil.
require "rspec/core"

def pairing(old, new) = Rex::Diff.__matches__(old.split, new.split)

p pairing("a b c", "a b c")
p pairing("a b x", "a b")
p pairing("a b c d e", "a x c y e")
p pairing("a b c", "b a c")
p pairing("a b", "c d")
