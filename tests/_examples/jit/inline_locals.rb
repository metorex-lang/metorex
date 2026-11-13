# Locals read and written in compiled code, each method run often enough to
# be compiled: Integers and nil, which the machine code copies itself, and
# Strings and Arrays, a local not assigned yet, and a local a block shares,
# which it leaves to the interpreter.
def counted(times)
  total = 0
  count = 0
  while count < times
    total += count
    count += 1
  end
  total
end

def changing_kinds(times)
  held = 0
  count = 0
  while count < times
    held = count.even? ? count : "odd #{count}"
    count += 1
  end
  held
end

def read_before_written(times)
  seen = []
  count = 0
  while count < times
    later = count if count.odd?
    seen << later
    count += 1
  end
  seen.first(3)
end

def shared_with_a_block(times)
  total = 0
  add = proc { |amount| total += amount }
  count = 0
  while count < times
    add.call(1)
    total += 1
    count += 1
  end
  total
end

def nil_and_back(times)
  value = nil
  count = 0
  while count < times
    value = value.nil? ? count : nil
    count += 1
  end
  value
end

results = []
100.times do
  results << [
    counted(50),
    changing_kinds(51),
    read_before_written(5),
    shared_with_a_block(10),
    nil_and_back(7)
  ]
end
results.uniq.each { |result| p result }
