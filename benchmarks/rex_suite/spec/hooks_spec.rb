RSpec.describe "a ledger" do
  before { @entries = [] }
  after { @entries.clear }
  around { |example| example.run }

  500.times do |index|
    it "records entry #{index}" do
      @entries << index
      expect(@entries).to eq([index])
    end
  end
end
