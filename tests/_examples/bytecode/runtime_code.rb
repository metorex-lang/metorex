# Code handed over as a String runs where it is handed over.
total = 1
eval("total += 1")
p total
p eval("[total, self.to_s]")

def leave_through_eval
  eval("return :left")
  :stayed
end
p leave_through_eval

class Account
  def initialize
    @balance = 10
  end
end
account = Account.new
p account.instance_eval("@balance + total")
Account.class_eval("def balance = @balance")
p account.balance
Account.module_eval <<~CODE
  def deposit(amount)
    @balance += amount
  end
CODE
p account.deposit(5)
Account.define_method(:doubled) { @balance * 2 }
p account.doubled

def binding_with_rate
  rate = 3
  binding
end
held = binding_with_rate
p held.eval("rate * 2")
held.eval("rate = 4")
p held.local_variable_get(:rate)

sequence = RubyVM::InstructionSequence.compile("[1, 2].sum + 3")
p sequence.eval

trace_var(:$watched, "$seen = $watched * 10")
$watched = 2
p $seen
untrace_var(:$watched)
