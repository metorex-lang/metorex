# include and extend are methods, so a program may name a local after one,
# as Action Controller's params wrapper and Active Record's index reader do.
include = [1].map(&:to_s)
p include
using, include, where = ["btree", "id", nil]
p [using, include, where]
extend = 2
p extend
