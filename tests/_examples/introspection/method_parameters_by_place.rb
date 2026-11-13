# Each kind of parameter a method takes, bound and read back, with the
# locals the method lists, each method run often enough to be compiled.
def required(first, second)
  [first, second, binding.local_variables]
end

def defaulted(first, second = first * 2, third = proc { second })
  sum = first + second
  [sum, third.call, binding.local_variables]
end

def default_reading_itself(value = value)
  value
end

def splatted(first, *rest, last)
  [first, rest, last]
end

def splatted_with_default(first, middle = :middle, *rest, last)
  [first, middle, rest, last]
end

def repeated(_, _, other)
  [_, other]
end

def keywords(first, size: 1, **others, &block)
  [first, size, others, block&.call]
end

def destructured((left, right), after)
  [left, right, after]
end

def splat_among_many_locals(*given)
  given = given.dup
  a1 = 1; a2 = 2; a3 = 3; a4 = 4; a5 = 5; a6 = 6; a7 = 7; a8 = 8; a9 = 9
  b1 = 1; b2 = 2; b3 = 3; b4 = 4; b5 = 5; b6 = 6; b7 = 7; b8 = 8; b9 = 9
  given + [a1 + b9]
end

def reassigned(value)
  value += 1
  value
end

Recorder = Class.new do
  outer = 10
  define_method(:closed_over) do |amount|
    outer += amount
    outer
  end
end

recorder = Recorder.new
results = []
100.times do
  results << [
    required(1, 2),
    defaulted(1),
    defaulted(1, 5, proc { :given }),
    default_reading_itself,
    splatted(1, 2, 3, 4),
    splatted_with_default(1, 2),
    repeated(:a, :b, :c),
    keywords(1, size: 3, color: :red) { :block },
    destructured([1, 2], 3),
    reassigned(4),
    splat_among_many_locals(1, 2)
  ]
end
results.uniq.each { |result| p result }
p recorder.closed_over(1)
