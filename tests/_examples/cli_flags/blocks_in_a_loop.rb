# Blocks written inside a loop, run once per pass: one yielded to, and ones
# handed to Integer#times, Array#each, and Array#map. The passes are counted
# from the environment.
def twice
  yield
  yield
end

def run(times)
  count = 0
  index = 0
  while index < times
    twice { count += 1 }
    2.times { |step| count += step }
    [1, 2].each { |number| count += number }
    count += [3].map { |number| number * 2 }.first
    index += 1
  end
  count
end

p run(Integer(ENV.fetch("PASSES", "10")))
