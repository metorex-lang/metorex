# Under -n, Kernel answers chomp, chop, sub and gsub as module functions and
# carries them as private instance methods.
print(Kernel.public_methods(false).include?(:chomp), " ")
print(Kernel.private_instance_methods(false).include?(:sub), "\n")
