describe "tagged examples" do
  it "runs fast", :fast do
    expect(1).to eq(1)
  end

  it "runs slow", :slow do
    expect(2).to eq(2)
  end

  it "has a priority", priority: :high do
    expect(3).to eq(3)
  end

  it "has a version", version: 3 do
    expect(4).to eq(4)
  end

  context "a fast group", :fast do
    it "inherits the tag" do
      expect(5).to eq(5)
    end
  end
end
