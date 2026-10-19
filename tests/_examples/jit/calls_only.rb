# Methods whose instructions all call into Rust, run often enough to be
# compiled.
TEXT = "done"
def first = "<#{TEXT}>"
def second = [TEXT, TEXT]
def third = { TEXT => TEXT }

answers = []
count = 0
while count < 1000
  answers << first << second << third
  count += 1
end
p answers.uniq
