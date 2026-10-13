def each_doubled(transform = proc { |value| yield value * 2 })
  [1, 2, 3].map { |value| transform.call(value) }
end

p each_doubled { |value| value + 1 }
p each_doubled(proc { |value| value })

def first_given(value = yield)
  value
end

p first_given { :from_the_block }
p first_given(:passed)

def block_from_default(given = block_given?, &block)
  given
end

p block_from_default {}
p block_from_default
