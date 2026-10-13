RSpec.shared_examples "a list" do
  it "is countable" do
    expect(subject).to respond_to(:count)
  end
end

feature "a feature group" do
  scenario "runs as an example with metadata" do |example|
    expect([example.metadata[:slow], example.metadata[:type]]).to eq([true, :feature])
  end
end

describe "hooks registered in the configuration" do
  it "run in RSpec's order" do
    HOOK_ORDER.clear
  end

  it "ran the after hooks in order for the example before" do
    expect(HOOK_ORDER).to eq(%i[prepend_after after append_after prepend_before before append_before])
  end
end

describe "settings added by a suite" do
  it "read their defaults" do
    expect([RSpec.configuration.api_host, RSpec.configuration.api_host?]).to eq(["localhost", true])
  end

  it "answer nil and false when unset" do
    expect([RSpec.configuration.retries, RSpec.configuration.retries?]).to eq([nil, false])
  end
end

describe "the current example" do
  it "is exposed under the configured name" do |example|
    expect(current_spec).to equal(example)
  end
end

describe "a group with database metadata", :database do
  it "runs the hook defined once matching metadata appeared" do
  end
end

describe Array do
  subject { [1, 2] }

  it_has_behavior "a list"
end

describe "docstrings" do
  it "are_formatted_by_the_configured_block" do
  end
end
