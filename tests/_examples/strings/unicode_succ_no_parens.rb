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
walk = "ঙ".upto "香"
p walk.to_a.size
walk = "aa".upto "b"
p walk.to_a
walk = "1".upto "003"
p walk.to_a
walk = "9".upto "11"
p walk.to_a
walk = "b".upto "a"
p walk.to_a
walk = "aa".upto "ab"
p walk.to_a
