# Each form of call a body compiles, printed so the bytecode engine's answers
# can be compared with the tree walker's and Ruby's.

class Recorder
  def record(first, second = :default, *rest, key: :none, **others, &block)
    [first, second, rest, key, others, block ? block.call(first) : :no_block]
  end

  def each_twice
    yield 1
    yield 2
    :finished
  end

  def -@
    :negated
  end

  def +@
    :positive
  end
end

def positional_and_splats
  recorder = Recorder.new
  values = [2, 3, 4]
  options = { key: :given, extra: 1 }
  [
    recorder.record(1),
    recorder.record(1, *values),
    recorder.record(1, key: :named),
    recorder.record(1, **options),
    recorder.record(1, 2) { |number| number * 10 },
    recorder.record(1, &:to_s),
  ]
end

def blocks_and_breaks
  recorder = Recorder.new
  seen = []
  finished = recorder.each_twice { |number| seen << number }
  broken = recorder.each_twice { |number| break number * 100 }
  [seen, finished, broken, [1, 2, 3].map { |number| number + 1 }]
end

def safe_navigation
  missing = nil
  present = "text"
  [missing&.length, present&.length, missing&.upcase(1, 2)]
end

def bare_calls
  helper(3) + helper(4) { |value| value }
end

def helper(value)
  block_given? ? yield(value * 2) : value
end

def kernel_calls
  [Integer("12"), Kernel::Float("1.5"), format("%03d", 7), Array(nil), rand(1)]
end

def operators
  recorder = Recorder.new
  [-recorder, +recorder, +3, !nil, 2 ** 3, 5 <=> 4, 6 === 6]
end

def patterns
  matched = ({ name: "a", size: 2 } in { name: String })
  { name: "b", size: 3 } => { size: }
  [matched, size, (5 in Integer), (5 in String)]
end

def definitions
  [defined?(String), defined?(@missing), defined?(puts), defined?(zzz)]
end

p positional_and_splats
p blocks_and_breaks
p safe_navigation
p bare_calls
p kernel_calls
p operators
p patterns
p definitions
