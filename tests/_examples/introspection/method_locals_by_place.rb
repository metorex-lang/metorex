# A method's locals as blocks, bindings, and recursion see them, each
# method run often enough to be compiled.
def read_before_assignment
  seen = seen
  seen
end

def shared_with_a_later_block
  total = 1
  add = proc { total += 1 }
  add.call
  total
end

def block_in_its_own_assignment
  holder = proc { holder }
  holder.call.equal?(holder)
end

def set_through_a_binding
  value = 1
  binding.local_variable_set(:value, 5)
  value
end

def binding_taken_before_assignment
  taken = binding
  later = 9
  [taken.local_variables, taken.local_variable_get(:later)]
end

def parameters_then_locals(first, second = 2)
  third = first + second
  binding.local_variables + [third]
end

def factorial(number)
  product = number
  product *= factorial(number - 1) if number > 1
  product
end

results = []
100.times do
  results << [
    read_before_assignment,
    shared_with_a_later_block,
    block_in_its_own_assignment,
    set_through_a_binding,
    binding_taken_before_assignment,
    parameters_then_locals(1),
    factorial(5)
  ]
end
results.uniq.each { |result| p result }
