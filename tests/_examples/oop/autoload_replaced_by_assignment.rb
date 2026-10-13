autoload :Ledger, "ledger_never_loaded"
Ledger = 5
p Object.autoload?(:Ledger), Ledger
module Books
  autoload :Entry, "entry_never_loaded"
  Entry = 3
end
p Books.autoload?(:Entry), Books::Entry
autoload :Receipt, "receipt_never_loaded"
Object.const_set(:Receipt, 7)
p Object.autoload?(:Receipt)
