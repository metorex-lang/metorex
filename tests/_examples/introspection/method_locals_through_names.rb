# A method's locals reached by name: through eval, a binding, and the
# blocks written in the method, each method run often enough to be
# compiled.
def many_locals
  a1 = 1; a2 = 2; a3 = 3; a4 = 4; a5 = 5; a6 = 6; a7 = 7; a8 = 8; a9 = 9
  b1 = 1; b2 = 2; b3 = 3; b4 = 4; b5 = 5; b6 = 6; b7 = 7; b8 = 8; b9 = 9
  taken = binding
  [taken.local_variable_get(:b9), taken.local_variables.size, a1 + a9 + b1 + b8]
end

def written_through_eval
  count = 1
  eval("count += 10")
  count
end

def reserved_read_through_a_binding
  taken = binding
  before = taken.local_variable_get(:pending)
  pending = 3
  [before, taken.local_variable_defined?(:pending), pending]
end

def reserved_set_through_a_binding
  binding.local_variable_set(:pending, 4)
  pending = pending.to_i + 1
  pending
end

def blocks_read_and_write_by_name
  total = 0
  [1, 2, 3].each { |number| total += number }
  [total, [4].map { |number| number + total }]
end

results = []
100.times do
  results << [
    many_locals,
    written_through_eval,
    reserved_read_through_a_binding,
    reserved_set_through_a_binding,
    blocks_read_and_write_by_name
  ]
end
results.uniq.each { |result| p result }
