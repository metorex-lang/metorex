# Two loops of 100 turns: one of Integer operations, which machine code runs
# itself, and one of calls, which it would leave to Rust.
def add_up
  total = 0
  count = 0
  while count < 100
    total += count
    count += 1
  end
  total
end

def call_around
  words = []
  count = 0
  while count < 100
    words.push(count.to_s.upcase.strip.freeze)
    count += 1
  end
  words.size
end

p add_up
p call_around
