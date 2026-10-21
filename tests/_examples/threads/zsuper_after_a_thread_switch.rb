class Ledger
  def record(entry, amount)
    [entry, amount]
  end
end

# Signals that it entered `record`, then waits before calling `super`, so the
# other thread runs its own `record` in between.
class WaitingLedger < Ledger
  def initialize(signal, wait_on)
    @signal = signal
    @wait_on = wait_on
  end

  def record(entry, amount)
    @signal.push(:entered)
    @wait_on.pop
    super
  end
end

worker_entered = Queue.new
worker_go = Queue.new
main_entered = Queue.new
main_go = Queue.new

worker = Thread.new { WaitingLedger.new(worker_entered, worker_go).record(:rent, 1200) }
helper = Thread.new do
  worker_entered.pop
  main_entered.pop
  worker_go.push(:go)
  worker.join
  main_go.push(:go)
end
main_result = WaitingLedger.new(main_entered, main_go).record(:water, 45)
helper.join
p worker.value
p main_result
