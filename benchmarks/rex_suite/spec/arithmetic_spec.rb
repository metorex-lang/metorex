RSpec.describe "arithmetic" do
  500.times do |index|
    it "adds one to #{index}" do
      expect(index + 1).to eq(index.succ)
    end
  end
end
