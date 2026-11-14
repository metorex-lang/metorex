# A hash keeps a frozen copy of a String key that is not frozen, and two
# hashes handed equal keys share one copy. A key carrying an instance
# variable of its own gets a copy of its own, and a frozen key is kept.
text = rand.to_s
first = {}
second = {}
first[+text] = 1
second[+text] = 2
p first.keys[0].frozen?, first.keys[0].equal?(second.keys[0])

marked = +text
marked.instance_variable_set(:@note, 1)
third = {}
third[marked] = 3
p third.keys[0].equal?(first.keys[0]), third.keys[0].equal?(marked)

frozen = text.dup.freeze
fourth = {}
fourth[frozen] = 4
p fourth.keys[0].equal?(frozen)
