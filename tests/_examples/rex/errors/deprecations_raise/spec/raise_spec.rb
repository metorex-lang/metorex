RSpec.configure(&:raise_errors_for_deprecations!)

describe "deprecations raised as errors" do
  it "fails where the deprecated call is" do
    counter = [0]
    expect(counter.push(1)).not_to change { counter.size }
  end

  it "passes without one" do
    expect(1).to eq(1)
  end
end
