TOPLEVEL_BINDING.eval("class Ledger; end")
def define_through = TOPLEVEL_BINDING.eval("class Tally; end")
define_through
p TOPLEVEL_BINDING.local_variables.include?(:Ledger)
p TOPLEVEL_BINDING.eval("Tally")
begin
  TOPLEVEL_BINDING.local_variable_defined?(:Ledger)
rescue NameError => error
  p error.message.start_with?("wrong local variable name 'Ledger' for #<Binding:0x")
end
