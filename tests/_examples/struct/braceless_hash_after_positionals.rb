Runner = Struct.new(:all_ids, :always_failures, :dependent_failures)

runner = Runner.new(%w[a b], [], "b" => %w[a])
p runner.all_ids
p runner.always_failures
p runner.dependent_failures

begin
  Runner.new("b" => %w[a])
rescue ArgumentError => error
  puts error.message
end

Point = Struct.new(:x, :y)
p Point.new(x: 1, y: 2).x
