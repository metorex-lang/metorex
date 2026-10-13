describe "include with a count" do
  it "passes once" do
    expect([1, 2, 3]).to include(2).once
  end

  it "passes at least twice" do
    expect("banana").to include("a").at_least(2).times
  end

  it "fails when found more often than exactly" do
    expect([1, 1, 2]).to include(1).exactly(1).times
  end

  it "fails when found fewer times than at least" do
    expect("apple").to include("p").at_least(3).times
  end

  it "fails outside a range" do
    expect([5, 5, 5, 5]).to include(5).at_least(1).at_most(2).times
  end

  it "fails negated when the count matches" do
    expect(%w[a b a]).not_to include("a").twice
  end

  it "describes itself" do
    expect(include(3).thrice.description).to eq("include 3 thrice")
  end

  it "refuses a count with two expected items" do
    expect([1, 2]).to include(1, 2).once
  end

  it "refuses two at_least constraints" do
    expect { include(1).at_least(1).at_least(2) }.to raise_error(ArgumentError, "Multiple at_least constraints are not supported")
  end
end
