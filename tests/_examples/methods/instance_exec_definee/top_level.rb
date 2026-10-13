class Ledger
end

ledger = Ledger.new
ledger.instance_exec { eval %{def balance = 10} }
p ledger.singleton_methods
p Object.private_method_defined?(:balance)

ledger.instance_exec { load File.join(__dir__, "loaded_helper.rb") }
p ledger.singleton_methods
p Object.private_method_defined?(:loaded_helper)
