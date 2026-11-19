# An integer raised to a negative power is the Rational one over the
# positive power, while 1 and -1 stay integers, as crass's number parsing
# relies on.
p [2**-3, (-2)**-3, 10**-2, 2.pow(-2), (2**70)**-1]
p [1**-1, (-1)**-1, (-1)**-2, 10**-0]
p (12 + 5 * 10**-1) * 10**2
