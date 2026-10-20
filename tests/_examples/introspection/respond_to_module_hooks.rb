# A module or class has the hooks Module and Class define privately, such as
# `method_added` and `inherited`, which `respond_to?` reports only when
# asked about private methods. A class has none of the hooks Class removes.
mod = Module.new
klass = Class.new

p mod.respond_to?(:method_added, true)
p mod.respond_to?(:method_added)
p mod.respond_to?(:inherited, true)
p klass.respond_to?(:inherited, true)
p klass.respond_to?(:append_features, true)
p klass.respond_to?(:extended, true)
