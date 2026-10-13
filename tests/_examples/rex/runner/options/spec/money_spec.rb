describe Money do
  it "adds cents" do
    expect(Money.new(150) + Money.new(250)).to eq(Money.new(400))
  end

  it "formats dollars" do
    expect(Money.new(1999).to_s).to eq("$19.99")
  end

  context "with zero" do
    it "formats zero" do
      expect(Money.new(0).to_s).to eq("$0.00")
    end
  end
end
