Rex.configure { |config| config.rspec_constant = false }

module RSpec
  def self.owner
    "the suite"
  end
end

Rex.describe "an RSpec constant the suite defines" do
  it "is the suite's own module" do
    expect(RSpec.owner).to eq("the suite")
  end

  it "leaves Rex under its own name" do
    expect(Rex.configuration.rspec_constant?).to be(false)
  end
end
