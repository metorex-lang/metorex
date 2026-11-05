# A pattern can open a method's body on the line its parameters end.
def escaping_pattern(str) /([#{Regexp.escape(str)}])/n end
def other_pattern(str) /([^#{Regexp.escape(str)}])/n end
def halved(value) value / 2 end
def nested((first, second)) /#{first}#{second}/ end

p escaping_pattern("ab")
p other_pattern("c")
p halved(10)
p nested(["x", "y"])
p "a/b".split(/\//)
