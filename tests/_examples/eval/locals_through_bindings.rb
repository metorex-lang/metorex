# Locals a method body assigns, read and written through a Binding, eval,
# and local_variables.

def written_through_a_binding
  count = 1
  binding.local_variable_set(:count, 2)
  count
end

def read_through_a_binding
  count = 3
  binding.local_variable_get(:count)
end

def listed
  first = 1
  second = first + 1
  local_variables
end

def written_by_eval
  count = 4
  binding.eval("count = count + 1")
  count
end

def reserved_before_assignment
  early = local_variables
  late = 1
  early
end

def added_by_eval
  binding.eval("added = 6")
  binding.local_variable_defined?(:added)
end

def summed_in_a_block
  total = 0
  [1, 2, 3].each { |number| total = total + number }
  total
end

def written_through_a_binding_in_a_block
  count = 1
  [8].each { |number| binding.local_variable_set(:count, number) }
  count
end

def block_local_kept_from_the_method
  [1, 2].map { |number| inner = number * 10 }
  binding.local_variable_defined?(:inner)
end

p written_through_a_binding
p read_through_a_binding
p listed
p written_by_eval
p reserved_before_assignment
p added_by_eval
p summed_in_a_block
p written_through_a_binding_in_a_block
p block_local_kept_from_the_method
