RSpec.describe "contain_exactly" do
  it "pairs items that match more than one expectation" do
    expect(["fool", "food"]).to contain_exactly(/foo/, a_string_ending_with("l"))
  end

  it "reports the fewest unpaired items" do
    expect([1, 2, 3, 4]).to contain_exactly(a_value < 3, a_value < 2, 5, 6)
  end

  it "reports items that match nothing" do
    expect(%w[web db cache]).to contain_exactly("web", "queue", a_string_starting_with("c"))
  end

  it "fails a value it cannot turn into an array" do
    expect(nil).to contain_exactly(1)
  end

  it "passes items in any order" do
    expect([3, 1, 2]).to match_array([1, 2, 3])
  end
end
