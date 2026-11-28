# A program that asks whether RSpec is defined before it requires it is told
# it is not, as MRI tells it.
p defined?(RSpec)
p defined?(::RSpec)
p defined?(Object::RSpec)
