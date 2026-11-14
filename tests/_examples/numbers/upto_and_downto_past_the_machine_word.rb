# `upto` and `downto` walk across the end of the machine word and between
# bignums, as concurrent-ruby's AtomicReference spec counts past 2**63.
near = 2**63 - 2
near.upto(near + 3) { |value| print value, " " }
puts
low = -(2**63) + 1
low.downto(low - 2) { |value| print value, " " }
puts
big = 2**64
big.downto(big - 1) { |value| print value, " " }
puts
p (big - 1).upto(big).to_a, big.downto(big - 1).size
p near.upto(near + 3).size, big.upto(big - 1).size
p (big + 1).downto(big - 0.5).to_a
p(big.upto(big + 5) { |value| break value if value > big + 1 })
