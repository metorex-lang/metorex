# A call's keyword values are worked out before its &block argument is
# handed over, so a method a keyword value calls does not take the block,
# as in Active Record's `log`, whose `transaction:` value calls methods.
class Instrumenter
  def instrument(name, payload = {})
    block_given? ? yield(payload) : :no_block
  end
end

class Ledger
  def instrumenter = Instrumenter.new
  def current_month = "2026-11"

  def log(sql, &block)
    instrumenter.instrument(
      "sql.ledger",
      sql:   sql,
      month: current_month,
      &block
    )
  end

  def log_without_parentheses(sql, &block)
    instrumenter.instrument "sql.ledger", sql: sql, month: current_month, &block
  end
end

ledger = Ledger.new
p ledger.log("SELECT 1") { |payload| payload }
p ledger.log_without_parentheses("SELECT 2") { |payload| payload[:month] }
p ledger.log("SELECT 3")
