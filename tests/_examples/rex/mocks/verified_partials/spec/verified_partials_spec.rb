class Ledger
  def record(entry, amount = 0) = [entry, amount]
  def self.open = new
end

describe "verified partial doubles" do
  it "rejects a method the class does not define" do
    allow(Ledger).to receive(:close)
  end

  it "rejects the wrong number of arguments on a real object" do
    ledger = Ledger.new
    allow(ledger).to receive(:record)
    ledger.record
  end

  it "accepts a method the class defines" do
    allow(Ledger).to receive(:open).and_return(:opened)
    expect(Ledger.open).to eq(:opened)
  end
end
