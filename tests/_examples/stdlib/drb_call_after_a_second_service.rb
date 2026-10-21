require "drb/drb"

class Ledger
  def balance = 75
end

ledger_service = DRb.start_service("druby://127.0.0.1:0", Ledger.new)
DRb.start_service("druby://127.0.0.1:0")
puts DRbObject.new_with_uri(ledger_service.uri).balance
DRb.stop_service
ledger_service.stop_service
