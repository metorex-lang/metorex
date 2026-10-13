Ledger = Struct.new(:entries) do
  def balance
    entries.sum
  end
end

RSpec.describe Ledger do
  subject(:ledger) { Ledger.new([100, -40]) }

  it "sums its entries" do
    expect(ledger.balance).to eq(60)
  end

  it "stubs only methods the object has" do
    expect { allow(ledger).to receive(:total) }.to raise_error(RSpec::Mocks::MockExpectationError)
  end

  it "describes a chained custom matcher with its chain" do
    expect(ledger).to have_entries.counting(2)
  end

  it "has no top-level describe" do
    expect(TOPLEVEL_BINDING.receiver.respond_to?(:describe, true)).to be(false)
  end
end

RSpec::Matchers.define :have_entries do
  match { |actual| actual.entries.size == @count }
  chain(:counting) { |count| @count = count }
end
