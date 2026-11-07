def count_down(remaining) = remaining.zero? ? :bottom : count_down(remaining - 1)
p count_down(1500)

def wrapped(remaining) = remaining.zero? ? :bottom : [remaining].map { |left| wrapped(left - 1) }.first
p wrapped(1000)
