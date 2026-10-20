# `def (Outer::Host).name` defines a singleton method on the module the
# constant path names.
module Outer
  module Host; end
end

def (Outer::Host).label = "host"

p Outer::Host.label
p Outer::Host.singleton_methods
p Object.private_method_defined?(:label)
