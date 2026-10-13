class Ledger
  def initialize
    @entries = []
  end

  def record(amount)
    raise ArgumentError, "an entry needs an amount" if amount.zero?

    @entries << amount
    self
  end

  def balance = @entries.sum

  def overdrawn? = balance.negative?
end
