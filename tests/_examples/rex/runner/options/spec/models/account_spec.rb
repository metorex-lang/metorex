require "account"

describe Account do
  it "starts empty" do
    expect(Account.new.balance).to eq(Money.new(0))
  end

  it "rounds wrong" do
    expect(Account.new.balance.to_s).to eq("$0.01")
  end
end
