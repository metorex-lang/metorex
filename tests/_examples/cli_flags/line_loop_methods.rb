# Under -n, Kernel answers chomp, chop, sub and gsub as module functions and
# carries them as private instance methods.
functions = Kernel.public_methods(false)
carried = Kernel.private_instance_methods(false)
print(functions.include?(:chomp), " ")
print(carried.include?(:sub), "\n")
