# A heredoc may be named with digits alone, as Tilt's Etanni templates name
# theirs after a hash of the template.
out = [<<12345.chomp]
first line
12345
out << <<0042
second line
0042
p out
p(<<99.upcase)
  third
99
