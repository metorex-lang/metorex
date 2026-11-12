# Native calls answered in compiled code without the native tables, each
# run in a loop often enough to be compiled, beside the values and
# arguments those calls leave to the tables.
def answers(times)
  seen = []
  count = 0
  while count < times
    seen << yield
    count += 1
  end
  seen.uniq
end

def nil_checks(value, times)
  seen = []
  count = 0
  while count < times
    seen << value.nil?
    count += 1
  end
  seen.uniq
end

def array_readers(values, times)
  seen = []
  count = 0
  while count < times
    seen << [values.size, values.length, values.empty?, values.first, values.last]
    count += 1
  end
  seen.uniq
end

def shifts(value, amount, times)
  seen = []
  count = 0
  while count < times
    seen << [value << amount, value >> amount]
    count += 1
  end
  seen.uniq
end

def sizes(value, times)
  seen = []
  count = 0
  while count < times
    seen << value.size
    count += 1
  end
  seen.uniq
end

p nil_checks(nil, 100)
p nil_checks(0, 100)
p array_readers([], 100)
p array_readers([3, 4, 5], 100)
p sizes("text", 100)
p sizes({ a: 1 }, 100)
p shifts(5, 3, 100)
p shifts(-8, 1, 100)
p shifts(1, 62, 100)
p shifts(3, 62, 100)
p shifts(1, 63, 100)
p shifts(1, -2, 100)
p shifts(1 << 70, 2, 100)
p answers(100) { [7, 8].first(1) }
appended = []
p answers(100) { (appended << 1).size }.size
