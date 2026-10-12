# A Binding taken in a block reports the implicit parameters that block
# takes, `it` or `_1` to `_9`, and not those of a block around it.
p(proc { _2; binding.implicit_parameters }.call(:north, :south))
p(proc { it; binding.implicit_parameter_get(:it) }.call(:east))
p(proc { _1; binding.implicit_parameter_defined?(:_1) }.call(:west))
p(proc { binding.implicit_parameter_defined?(:it) }.call(:west))
p(proc { _1; proc { binding.implicit_parameters }.call }.call(:up))

begin
  proc { binding.implicit_parameter_get(:_1) }.call
rescue NameError => problem
  p problem.message.start_with?("implicit parameter '_1' is not defined for")
end
begin
  binding.implicit_parameter_get(:total)
rescue NameError => problem
  p problem.message
end

# A Symbol spelled like a numbered parameter is a Symbol, and a block that
# names one takes no parameter.
p(proc { :_1 }.arity)
p([3, 4].map { false ? :_1 : _1 })
