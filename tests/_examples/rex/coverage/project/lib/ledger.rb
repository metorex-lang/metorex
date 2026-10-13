class Ledger
  def initialize(entries)
    @entries = entries
  end

  def balance
    @entries.sum
  end

  def status
    if balance.positive?
      :credit
    elsif balance.zero?
      :even
    else
      :debit
    end
  end

  def largest
    @entries.max
  end

  def label
    @entries.empty? ? "empty" : "#{@entries.size} entries"
  end
end
