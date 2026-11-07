seen = [1, 2, 3].map do |n|
  begin
    raise ArgumentError if n == 2
  rescue ArgumentError
    next :skipped
  end
  n * 10
end
p seen
runner = proc do |value|
  begin
    Integer(value)
  rescue ArgumentError
    next :bad
  end
end
p runner.call("4"), runner.call("x")
def each_checked
  yield 1
  yield 2
end
collected = []
each_checked do |n|
  begin
    raise "no" if n == 1
  rescue
    next
  ensure
    collected << [:ensure, n]
  end
  collected << n
end
p collected
body = proc do
  begin
    raise ArgumentError
  rescue ArgumentError
    next :from_rescue
  end
  :after
end
p Object.new.instance_exec(&body), Object.new.instance_eval(&body)
plain = proc { next :plain; :after }
p Object.new.instance_exec(&plain)
