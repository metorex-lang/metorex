# coding: utf-8

literal = "rené"
relabeled = "ren\xC3\xA9".b.encode("utf-8", "utf-8")
p literal.encoding, relabeled.encoding, literal == relabeled, [relabeled].include?(literal)
p [relabeled].index(literal), [relabeled] == [literal], [relabeled] - [literal]
p({ relabeled => 1 }[literal], [relabeled, literal].uniq.size, [relabeled].delete(literal))
