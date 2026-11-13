# Indexing a local whose name is also a Kernel function's, beside the
# Kernel function written the same way where no local has the name.
def indexed_locals
  p = [10, 20, 30]
  puts = { "first" => 1 }
  format = "abcdef"
  [p[1], puts["first"], format[1, 3], format[-2..]]
end

def assigned_in_a_block
  rand = nil
  [1].each { rand = [7, 8] }
  rand[0]
end

def parameter_named_like_one(print)
  print[0]
end

results = []
100.times do
  results << [indexed_locals, assigned_in_a_block, parameter_named_like_one("xyz")]
end
results.uniq.each { |result| puts result.inspect }
p [:written, :out]
