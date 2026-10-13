RSpec.configure do |config|
  config.treat_symbols_as_metadata_keys_with_true_values = true
end

describe "deprecated usage" do
  5.times do |attempt|
    it "hands a block matcher a value, attempt #{attempt + 1}" do
      counter = [0]
      expect(counter.push(1)).not_to change { counter.size }
    end
  end
end
