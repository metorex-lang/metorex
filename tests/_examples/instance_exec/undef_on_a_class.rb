class Runner
  def self.autorun = :installed
end

Runner.instance_exec do
  undef :autorun
  def autorun = :replaced
end
p Runner.autorun

ledger = Object.new
def ledger.total = 75
ledger.instance_eval { undef total }
p ledger.respond_to?(:total)
