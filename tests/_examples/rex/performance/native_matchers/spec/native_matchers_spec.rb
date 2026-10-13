RSpec.describe "the matchers Rex matches natively" do
  it "match without running a matcher's matches?" do
    calls = 0
    trace = TracePoint.new(:call) { |tp| calls += 1 if tp.method_id == :matches? }
    trace.enable do
      expect(42).to eq(42)
      expect(:ledger).to be(:ledger)
      expect("open").to be_truthy
      expect(nil).to be_nil
      expect("invoice 1001").to match(/\d+/)
      expect([12, 30]).to include(30)
    end
    expect(calls).to eq(0)
  end

  it "report a failure as RSpec writes it" do
    expect { expect([12, 30]).to include(42) }.to raise_error(RSpec::Expectations::ExpectationNotMetError, "expected [12, 30] to include 42")
  end
end
