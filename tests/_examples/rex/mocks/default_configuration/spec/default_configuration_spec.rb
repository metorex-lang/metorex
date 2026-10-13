RSpec.describe "the mocks configuration a suite leaves alone" do
  let(:ledger) { Object.new }

  it "stubs a method the object does not have" do
    allow(ledger).to receive(:close).and_return(:closed)
    expect(ledger.close).to eq(:closed)
  end

  it "runs stub with a deprecation" do
    ledger.stub(:balance).and_return(10)
    expect(ledger.balance).to eq(10)
  end

  it "lists both syntaxes" do
    expect(RSpec::Mocks.configuration.syntax).to eq([:should, :expect])
  end

  it "does not verify partial doubles" do
    expect(RSpec::Mocks.configuration.verify_partial_doubles?).to be false
  end

  it "adds the should syntax to a module" do
    host = Module.new
    RSpec::Mocks.configuration.add_stub_and_should_receive_to(host)
    expect(host.method_defined?(:should_receive)).to be true
  end
end
