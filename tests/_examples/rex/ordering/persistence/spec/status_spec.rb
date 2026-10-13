RSpec.configure do |config|
  config.example_status_persistence_file_path = ENV["STATUS_FILE"]
end

describe "an invoice" do
  it "lists the customer" do
    expect("Acme").to eq("Acme")
  end

  it "sums the lines" do
    expect(2 + 2).to eq(5)
  end

  it "formats the date" do
    expect("2026-10-12").to eq("2026-10-13")
  end

  it "waits for the currency rule" do
    skip "the currency rule"
  end
end
