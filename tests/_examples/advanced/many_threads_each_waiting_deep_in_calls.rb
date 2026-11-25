# Each thread counts how deeply it is nested on its own, so many threads
# waiting part-way down a chain of calls do not add up to a stack that is
# too deep, as a server's reader threads each wait inside a few dozen calls.
def descend(levels, gate)
  return gate.pop if levels.zero?
  descend(levels - 1, gate)
end

gate = Queue.new
threads = 150.times.map { Thread.new { descend(40, gate) } }
sleep 0.1 until threads.all? { |thread| thread.status == "sleep" }
150.times { |number| gate << number }
p threads.map(&:value).sort == (0...150).to_a

def bottomless(level) = bottomless(level + 1)

deep = Thread.new do
  bottomless(0)
rescue SystemStackError => error
  error.class
end
p deep.value
