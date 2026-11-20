# Modules defined with `module` inside an anonymous module have no permanent
# name, so each one can take a temporary name. Binding the outer module to a
# constant later replaces those temporary names with permanent ones.
outer = Module.new
module outer::Middle; module Inner; end; end

outer::Middle::Inner.set_temporary_name("inner")
p outer::Middle::Inner.name

outer::Middle.set_temporary_name("middle")
p outer::Middle.name
p outer::Middle::Inner.name

Named = outer
p outer::Middle.name
p outer::Middle::Inner.name

holder = Module.new
class holder::Widget; end
holder::Widget.set_temporary_name("widget")
p holder::Widget.name
Holder = holder
p holder::Widget.name
