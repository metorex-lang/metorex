describe Array do
  subject { [3, 1, 2] }

  it_behaves_like "a collection", 3

  it_should_behave_like "a sortable collection" do
    let(:expected_order) { [1, 2, 3] }
  end

  context "when included in place" do
    include_examples "a collection", 4
  end
end

describe Hash do
  subject { { b: 2, a: 1 } }

  shared_examples "a lookup" do
    it "finds a key" do
      expect(subject[:a]).to eq(1)
    end
  end

  it_behaves_like "a lookup"
  it_behaves_like "a collection", 2
end

describe "a context with a logger", :logged do
  it "runs the context's before hook" do
    expect(logger).to eq(["started"])
  end
end

describe "a group including a context by name" do
  include_context "with a logger"

  it "has the context's let" do
    expect(logger).to eq(["started"])
  end
end

describe "metadata" do
  it "names its location", :fast, priority: :high do |example|
    expect(example.metadata).to include(
      file_path: "./spec/shared_spec.rb",
      line_number: 43,
      location: "./spec/shared_spec.rb:43",
      full_description: "metadata names its location",
      fast: true,
      priority: :high
    )
  end

  describe Array, :slow do
    it "inherits its group's metadata and described class" do |example|
      expect(example.metadata).to include(slow: true, described_class: Array, scoped_id: "5:2:1")
    end
  end
end

describe String do
  subject { "abc" }

  it_behaves_like "a collection", 5
end
