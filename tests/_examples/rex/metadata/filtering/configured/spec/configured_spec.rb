RSpec.configure do |config|
  config.filter_run_excluding broken: true
  config.filter_run_excluding version: ->(version) { version > 3 }
end

describe "configured filters" do
  it "runs" do
    expect(1).to eq(1)
  end

  it "is excluded as broken", :broken do
    raise "never runs"
  end

  it "runs on an old version", version: 2 do
    expect(2).to eq(2)
  end

  it "is excluded on a new version", version: 4 do
    raise "never runs"
  end
end
