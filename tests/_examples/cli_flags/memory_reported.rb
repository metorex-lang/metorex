# A program METOREX_MEMORY_REPORT measures: it keeps 50,000 strings and an
# instance of a class of its own alive until it ends.
class Ledger
  def initialize
    @entries = []
  end

  def record(entry) = @entries << entry

  def size = @entries.size
end

ledger = Ledger.new
50_000.times { |index| ledger.record("invoice #{index} due 2026-11-01") }
puts ledger.size
