# A block written once is compiled once, however many times the code that
# writes it runs.
def doubled(numbers) = numbers.map { |number| number * 2 }

doubled([1])
before = RubyVM.stat(:units_compiled)
100.times { doubled([1, 2]) }
p RubyVM.stat(:units_compiled) - before
