describe "a ledger" do
  it "balances" do
    expect(100 - 40).to eq(60)
  end

  it "rejects a negative entry" do
    expect(-5).to be_negative
  end
end

describe "a ledger written to a StringIO" do
  it "holds what the documentation formatter wrote so far" do
    expect(CAPTURED.string).to include("a ledger\n  balances\n  rejects a negative entry\n")
  end
end
