require "ripper"
require "pp"

# Assignments written as arguments, one of them nested in a group.
pp Ripper.sexp("a(b = c, (d, e = f), G = h)")
pp Ripper.sexp("foo a = 1, 2")
pp Ripper.sexp("x = 1, 2")
pp Ripper.sexp("foo(bar { y = 1, 2 })")
