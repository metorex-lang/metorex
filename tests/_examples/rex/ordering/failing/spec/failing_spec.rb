describe "an order total" do
  before(:context) { puts "before the context" }

  it "adds the line items" do
    expect(1 + 2).to eq(3)
  end

  it "applies the discount" do
    expect(10 - 2).to eq(9)
  end

  it "adds the tax" do
    expect(10 * 2).to eq(21)
  end

  it "rounds to cents" do
    expect(1.005.round(2)).to eq(1.01)
  end

  it "is pending until the rule is written" do
    pending "the shipping rule"
    expect(0).to eq(1)
  end
end
