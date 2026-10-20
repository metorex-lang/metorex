# Several blocks written in one scope, which close over the same locals
# unless a local is added or changes between them.
def run_each
  yield
end

seen = []
40.times do |round|
  first = -> { round }
  run_each { seen << first.call }
  later = round * 2
  second = -> { [round, later] }
  run_each { seen << second.call }
  later += 1
  seen << second.call
end
p seen.last(3)
p seen.size

def held_method = :held
called = 40.times.map do |round|
  held = round == 20 ? method(:held_method) : round
  reader = -> { held }
  value = reader.call
  value.is_a?(Method) ? value.call : value
end
p called[19..21]

counter = 0
adders = 40.times.map do
  one = -> { counter += 1 }
  two = -> { counter += 10 }
  one.call
  two.call
  [one, two]
end
adders.last.each(&:call)
p counter
