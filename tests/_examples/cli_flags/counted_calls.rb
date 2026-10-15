# A program METOREX_COUNTERS counts: 250 calls of a method, 250 of a class
# method, and a `while` loop that runs 250 times.
class Tally
  def self.start = new

  def bump(by) = by + 1
end

tally = Tally.start
step = 0
while step < 250
  tally.bump(step)
  Tally.start
  step += 1
end
puts step
