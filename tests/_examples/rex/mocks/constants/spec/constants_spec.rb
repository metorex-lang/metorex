module Billing
  RATE = 5

  class Gateway
    TIMEOUT = 30
  end
end

RSpec.describe "constant stubbing" do
  it "replaces a defined constant" do
    stub_const("Billing::RATE", 7)
    expect(Billing::RATE).to eq(7)
  end

  it "defines an undefined constant and its parent modules" do
    stub_const("Reports::Monthly::LIMIT", 3)
    expect(Reports::Monthly::LIMIT).to eq(3)
  end

  it "hides a constant" do
    hide_const("Billing::RATE")
    expect(defined?(Billing::RATE)).to be_nil
  end

  it "transfers nested constants when asked" do
    stub_const("Billing::Gateway", Class.new, transfer_nested_constants: true)
    expect(Billing::Gateway::TIMEOUT).to eq(30)
  end

  it "reports what a stub replaced" do
    stub_const("Billing::RATE", 9)
    original = RSpec::Mocks::Constant.original("Billing::RATE")
    expect([original.stubbed?, original.previously_defined?, original.original_value]).to eq([true, true, 5])
  end

  it "refuses a name that is not a String" do
    stub_const(:RATE, 1)
  end

  it "puts every constant back after each example" do
    expect([Billing::RATE, defined?(Reports)]).to eq([5, nil])
  end
end
