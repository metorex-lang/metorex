# Branches and loops a body compiles, printed so the bytecode engine's
# answers can be compared with the tree walker's and Ruby's.

def branches(value)
  first = if value > 10
    :large
  elsif value > 5
    :medium
  else
    :small
  end
  second = unless value.even? then :odd else :even end
  third = if value.positive? then :positive end
  [first, second, third]
end

def loops
  count = 0
  seen = []
  while count < 10
    count += 1
    next if count.odd?
    break if count > 6
    seen << count
  end
  until count.zero?
    count -= 4
    count = 0 if count < 0
  end
  attempts = 0
  attempts += 1 while attempts < 3
  broken = while true
    break :carried
  end
  [seen, count, attempts, broken]
end

def redone
  passes = 0
  total = 0
  while passes < 2
    passes += 1
    total += 10
    redo if total == 10
  end
  [passes, total]
end

def for_loops
  sum = 0
  for number in [1, 2, 3]
    sum += number
  end
  [sum, number]
end

def cases(value)
  named = case value
  when Integer then :integer
  when "a", "b" then :letter
  when 1.5..2.5 then :range
  else :other
  end
  named
end

def block_controls
  doubled = [1, 2, 3, 4].map do |number|
    next 0 if number == 2
    number * 2
  end
  found = [5, 6, 7].each do |number|
    break number if number > 5
  end
  [doubled, found]
end

def conditions(line)
  $_ = line
  [(/needle/ ? :matched : :missing), (true && nil ? 1 : 2), (false || :right)]
end

p branches(12)
p branches(7)
p branches(3)
p loops
p redone
p for_loops
p [cases(1), cases("b"), cases(2.0), cases(:x)]
p block_controls
p conditions("a needle here")
