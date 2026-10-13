RSpec::Matchers.define :have_total do |expected|
  match do |actual|
    expect(actual.sum).to eq(expected)
  end

  failure_message do |actual|
    "expected #{actual.inspect} to total #{expected}, arguments #{expected_as_array.inspect}"
  end
end

describe "the should syntax" do
  it "passes with should" do
    [1, 2].should include(2)
  end

  it "passes with should_not" do
    5.should_not eq(4)
  end

  it "fails with should" do
    5.should eq(6)
  end

  it "fails with should_not" do
    5.should_not be > 4
  end

  it "runs the expect syntax alongside" do
    expect(3).to eq(3)
  end
end

describe "a matcher that calls expect" do
  it "passes" do
    expect([1, 2, 3]).to have_total(6)
  end

  it "fails with its own message" do
    expect([1, 2, 3]).to have_total(7)
  end
end
