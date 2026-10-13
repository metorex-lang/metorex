RSpec.describe "an invoice" do
  subject(:invoice) { { number: number, lines: lines } }

  let(:number) { 1001 }
  let(:lines) { [12, 30] }

  250.times do |index|
    context "with line #{index}" do
      let(:lines) { [12, 30, index] }

      it "sums its lines" do
        expect(invoice[:lines].sum).to eq(42 + index)
      end

      it "keeps its number" do
        expect(invoice[:number]).to eq(1001)
      end
    end
  end
end
