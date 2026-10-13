RSpec.configure { |config| config.order = :defined }

$initialized = 0
$initialize_trace = TracePoint.new(:call) { |tp| $initialized += 1 if tp.method_id == :initialize && tp.defined_class == Rex::Core::ExampleGroup }

RSpec.describe "a ledger" do
  before(:context) { $initialize_trace.enable }
  let(:total) { 42 }

  it("reads a let") { expect(total).to eq(42) }
  it("keeps RSpec's instance variables") { expect(instance_variables).to eq([:@__inspect_output, :@__memoized]) }
end

RSpec.describe "the ledger's examples" do
  it "were built without running ExampleGroup#initialize" do
    $initialize_trace.disable
    expect($initialized).to eq(0)
  end
end
