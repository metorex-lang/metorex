# RSpec loads Rex the first time the program names it.

RSpec.describe "a ledger" do
  it "sums its entries" do
    expect([12, 30].sum).to eq(42)
  end

  it "rounds a total" do
    expect(10.0 / 4).to eq(2)
  end
end
