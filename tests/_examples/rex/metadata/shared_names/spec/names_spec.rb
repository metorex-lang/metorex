module Countable
end

RSpec.shared_examples :countable do
  it "counts by a Symbol name" do
    expect(subject.count).to eq(2)
  end
end

RSpec.shared_examples Countable do
  it "counts by a Module name" do
    expect(subject.count).to eq(2)
  end
end

RSpec.shared_context "with a tagged default", :tagged do
  let(:default) { "from the tagged context" }
end

describe "shared group names" do
  subject { [1, 2] }

  it_behaves_like :countable
  include_examples Countable
end

describe "a group with the tag", :tagged do
  it "includes the context the tag triggers" do |example|
    expect([default, example.metadata[:tagged]]).to eq(["from the tagged context", true])
  end
end

describe "a block argument and a customization block" do
  shared_examples "a sum" do |left, right|
    it "adds its arguments" do
      expect(left + right).to eq(total)
    end
  end

  it_behaves_like "a sum", 2, 3 do
    let(:total) { 5 }
  end
end
