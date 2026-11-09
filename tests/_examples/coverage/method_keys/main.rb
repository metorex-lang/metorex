require "coverage"
Coverage.start(methods: true)
path = File.join(__dir__, "ledger.rb")
load path
ledger = Ledger.open
ledger.balance
ledger.balance
ledger.audited?
Coverage.result[path][:methods].sort_by { |key, _| [key[2], key[3]] }.each do |key, count|
  p [*key, count]
end
