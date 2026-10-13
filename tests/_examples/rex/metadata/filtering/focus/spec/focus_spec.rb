RSpec.configure do |config|
  config.filter_run_when_matching :focus
end

describe "focused runs" do
  it "is skipped while something is focused" do
    expect(1).to eq(1)
  end

  fit "runs because it is focused" do
    expect(2).to eq(2)
  end

  fdescribe "a focused group" do
    it "runs every example in it" do
      expect(3).to eq(3)
    end
  end
end
