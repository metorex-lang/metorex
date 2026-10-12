# `succ` steps a letter or digit outside ASCII to the next of its kind, and
# wraps one at the end of its run, carrying into the place to its left.
p "ঙ".succ
p "হ".succ
p "٩".succ
p "あ".succ
p "zz99".succ
# A byte that is not a character in its encoding steps as a byte.
p "(\xFF".b.succ
p "\xFF".b.succ
p "".succ

# `upto` stops once a step grows longer than the end, which bounds a walk
# through characters outside ASCII.
p "ঙ".upto("香").to_a.size
p "aa".upto("b").to_a
p "1".upto("003").to_a
p "9".upto("11").to_a
p "b".upto("a").to_a
p "aa".upto("ab").to_a
