RSpec.configure { |config| config.fail_if_no_examples = true }

RSpec.describe "a group with every example filtered out" do
  it "is slow", :slow do
  end
end
