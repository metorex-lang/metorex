RSpec.describe "a payment gateway" do
  let(:gateway) { double("gateway") }

  250.times do |index|
    it "charges card #{index}" do
      allow(gateway).to receive(:charge).with(index).and_return(:approved)
      expect(gateway.charge(index)).to eq(:approved)
    end

    it "notifies once for card #{index}" do
      expect(gateway).to receive(:notify).once
      gateway.notify
    end
  end
end
