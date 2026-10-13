RSpec.describe "monthly reports" do
  context "for a quiet month" do
    it "has no rows" do
      expect([]).to be_empty
    end
  end

  it "hands a block matcher a value" do
    counter = [0]
    expect(counter.push(1)).not_to change { counter.size }
  end
end

RSpec.describe "exports" do
  it "is skipped", skip: "the CSV format" do
  end
end
