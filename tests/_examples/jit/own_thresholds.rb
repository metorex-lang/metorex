# Two loops of 100 turns: one of Integer operations, which machine code runs
# itself, and one that builds a string, which it would leave to Rust.
def add_up
  total = 0
  count = 0
  while count < 100
    total += count
    count += 1
  end
  total
end

def build_up
  text = ""
  count = 0
  while count < 100
    text = "#{count}#{count}#{count}#{count}#{count}#{count}#{count}#{count}#{count}#{count}#{count}#{count}"
    count += 1
  end
  text.size
end

p add_up
p build_up
