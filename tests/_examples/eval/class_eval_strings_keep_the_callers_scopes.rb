Store = :top_level

class Object
  def scope_binding = class_eval("binding", __FILE__, __LINE__)
end

captured = BasicObject.scope_binding
p captured.eval("Store")
p captured.eval("Module.nesting")
begin
  BasicObject.class_eval("Store")
rescue NameError => error
  p error.message
end

group = Class.new do
  $from_a_block = proc { BasicObject.scope_binding.eval("Store") }
end
p group.new.instance_exec(&$from_a_block)
