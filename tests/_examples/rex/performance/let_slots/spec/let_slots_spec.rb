RSpec.describe "a let read more than once" do
  let(:total) { [12, 30].sum }

  it "is computed once and read from its slot after that" do
    lookups = 0
    trace = TracePoint.new(:call) { |tp| lookups += 1 if tp.method_id == :fetch_or_store }
    trace.enable { 3.times { total } }
    expect([total, lookups]).to eq([42, 1])
  end
end
