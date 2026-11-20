# A nested module that refers back to its outer module forms a cycle of
# constants. Naming the outer module renames each module in the cycle once.
outer = Module.new
outer::Inner = Module.new
outer::Inner::Back = outer

outer.set_temporary_name("outer")
p outer.name
p outer::Inner.name
p outer::Inner::Back.name

outer.set_temporary_name(nil)
p outer.name
p outer::Inner.name
