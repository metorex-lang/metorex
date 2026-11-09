p binding.respond_to?(:source_location)
p binding.respond_to?(:local_variable_set)
p binding.respond_to?(:eval)
p binding.method(:receiver).owner
