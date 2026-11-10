# Kernel's functions are its module functions too, so Kernel lists them as
# its own singleton methods, as RuboCop's Style/RedundantSelf reads
# `Kernel.methods(false)` to accept `self.open`.
listed = Kernel.methods(false)
p listed.include?(:open), listed.include?(:puts), listed.include?(:Integer), listed.include?(:format)
p Kernel.singleton_methods.include?(:sprintf), listed.include?(:pp), listed.include?(:initialize_copy)
