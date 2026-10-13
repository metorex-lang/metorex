RSpec.shared_examples "a collection" do
  it("reports its size") { expect(collection.size).to eq(3) }
  it("is not empty") { expect(collection).not_to be_empty }
end

RSpec.describe "collections" do
  250.times do |index|
    context "array #{index}" do
      let(:collection) { [index, index + 1, index + 2] }

      it_behaves_like "a collection"
    end
  end
end
