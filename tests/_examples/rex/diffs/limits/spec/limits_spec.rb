RSpec.configure do |config|
  config.diff_line_limit = 3
end

describe "diff limits" do
  it "diffs a value within the line limit" do
    expect("one\ntwo").to eq("one\nthree")
  end

  it "prints no diff past the line limit" do
    expect("a\nb\nc\nd").to eq("a\nb\nc\ne")
  end
end
