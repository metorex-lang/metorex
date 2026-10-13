RSpec.describe "the syntax a suite has not configured" do
  it "runs should with a deprecation" do
    [1, 2].should include(2)
  end

  it "runs should_not without a second deprecation" do
    5.should_not eq(4)
  end

  it "lists both syntaxes" do
    expect(RSpec::Expectations.configuration.syntax).to eq([:should, :expect])
  end

  it "keeps expect" do
    expect(RSpec::Expectations::Syntax.expect_enabled?).to be true
  end
end

RSpec.describe "a syntax the suite turns off" do
  around do |example|
    RSpec::Expectations.configuration.syntax = :expect
    example.run
  ensure
    RSpec::Expectations.configuration.syntax = [:should, :expect]
  end

  it "leaves objects without should" do
    expect(5).not_to respond_to(:should)
  end

  it "lists only expect" do
    expect(RSpec::Expectations.configuration.syntax).to eq([:expect])
  end
end

RSpec.describe "should added to one module" do
  let(:host) { Module.new }

  it "defines should on that module" do
    RSpec::Expectations.configuration.add_should_and_should_not_to(host)
    expect(host.method_defined?(:should_not)).to be true
  end
end
