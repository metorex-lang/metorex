true.instance_eval("def yes_label = :yes")
nil.instance_eval { def none_label = :none }
captured = true.instance_eval("binding")
captured.eval("def flag_label = :flag")
p TrueClass.instance_methods(false).include?(:flag_label)
p NilClass.instance_methods(false).include?(:none_label)
p true.yes_label, nil.none_label, true.flag_label
p false.respond_to?(:yes_label)
begin
  :status.instance_eval("def nope; end")
rescue TypeError => error
  p error.message
end
