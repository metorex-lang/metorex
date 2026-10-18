# Methods whose instructions are all calls, run often enough to be compiled.
def first = second
def second = third
def third = :done

answers = []
count = 0
while count < 1000
  answers << first
  count += 1
end
p answers.uniq
