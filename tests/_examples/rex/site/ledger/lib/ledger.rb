class Ledger
  def initialize
    @entries = []
  end

  def record(amount)
    if amount.zero?
      raise ArgumentError, "an entry needs an amount"
    end

    @entries << amount
    self
  end

  def balance = @entries.sum

  def overdrawn? = balance.negative?
end
