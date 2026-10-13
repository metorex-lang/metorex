RSpec.describe "invoices" do
  it "prints while it runs" do
    puts "rendering invoice 1001"
    expect(1).to eq(1)
  end

  it "fails on a total" do
    expect(10 + 5).to eq(16)
  end
end

RSpec.describe "payments" do
  before(:context) { raise "the gateway is down" }

  it "captures a card" do
  end
end

RSpec.describe "refunds" do
  it "waits on the refund rule" do
    pending "the refund rule"
    expect(1).to eq(2)
  end

  it "warns on stderr" do
    warn "refund queue is long"
  end
end
