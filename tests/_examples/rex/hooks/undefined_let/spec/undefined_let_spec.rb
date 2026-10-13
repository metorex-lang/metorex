describe "an account" do
  let(:account) { { balance: 100 } }
  let(:owner) { "Ada" }

  context "when nested" do
    let(:statement) { "monthly" }

    it "misspells a let" do
      expect(acount[:balance]).to eq(100)
    end
  end
end
