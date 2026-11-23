# The constructors of Time, Date and DateTime keep their arguments when an
# alias on the singleton class stands for them.
require "date"

Time.singleton_class.alias_method :original_new, :new
Time.singleton_class.alias_method :original_at, :at
Date.singleton_class.alias_method :original_civil, :civil
DateTime.singleton_class.alias_method :original_civil, :civil

p Time.original_new(2004, 11, 24, 1, 4, 44, "+00:00")
p Time.original_new(2004, 11, 24, 1, 4, 44).year
p Time.original_new("2000-12-31 23:59:59.56789", precision: 3).usec
p Time.original_at(0, in: "UTC")
p Date.original_civil(2004, 11, 24)
p DateTime.original_civil(2004, 11, 24, 1, 4, 44)
