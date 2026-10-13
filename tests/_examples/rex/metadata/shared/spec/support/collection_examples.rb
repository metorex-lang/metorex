RSpec.shared_examples "a collection" do |size|
  it "reports its size" do
    expect(subject.size).to eq(size)
  end

  it "is not empty" do
    expect(subject).not_to be_empty
  end
end

RSpec.shared_context "with a logger" do
  let(:logger) { [] }
  before { logger << "started" }
end

RSpec.shared_examples_for "a sortable collection" do
  it "sorts" do
    expect(subject.sort).to eq(expected_order)
  end
end
