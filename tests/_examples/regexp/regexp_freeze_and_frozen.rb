# A Regexp literal is frozen, Regexp.new makes one that is not, and freeze
# freezes it.
made = Regexp.new("a")
p /a/.frozen?
p made.frozen?
p made.freeze.equal?(made)
p made.frozen?
p Regexp.new("b").freeze.frozen?
part = "b+"
p(/a#{part}/.frozen?)
p(/a#{part}/o.frozen?)
p Regexp.union("a", "b").frozen?
