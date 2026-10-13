$lookups = 0

Rex::Core::MemoizedHelpers::ThreadsafeMemoized.class_exec do
  alias_method :rex_fetch_or_store, :fetch_or_store

  def fetch_or_store(key, &block)
    $lookups += 1
    rex_fetch_or_store(key, &block)
  end
end

RSpec.describe "a let whose memo lookup was redefined" do
  let(:total) { [12, 30].sum }

  it "reads every value through the redefined lookup" do
    3.times { total }
    expect([total, $lookups]).to eq([42, 4])
  end
end
