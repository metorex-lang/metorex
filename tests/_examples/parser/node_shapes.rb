# The node `parse` answers for each kind of statement and expression.
p parse('"a#{x}"').first
p parse("def f(a, b)\n  a\nend").first
p parse("while x\n  y\nend").first
p parse("for i in list\n  i\nend").first
p parse("raise Error").first
p parse("while x\n  next\n  break\nend").first["body"]
p parse("list[0]").first
p parse("-x").first
p parse("!x").first
