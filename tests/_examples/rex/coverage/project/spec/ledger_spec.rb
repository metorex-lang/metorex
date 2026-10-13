RSpec.describe Ledger do
  it "sums its entries" do
    expect(Ledger.new([5, -2]).balance).to eq(3)
  end

  it "is in credit with a positive balance" do
    expect(Ledger.new([5]).status).to eq(:credit)
  end

  it "labels a ledger with entries" do
    expect(Ledger.new([1, 2]).label).to eq("2 entries")
  end
end

RSpec.describe Fees do
  it "charges a small fee for a small amount" do
    expect(Fees.for(50)).to eq(1)
  end
end
