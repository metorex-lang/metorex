# A singleton method belongs to the one Proc, Method, or Binding it is
# defined on.
first = proc { 1 }
second = proc { 2 }
def first.label = "first"
p first.label
p second.respond_to? :label
p first.singleton_methods

taken = 1.method :succ
def taken.label = "succ"
p taken.label

held = binding
def held.label = "binding"
p held.label
