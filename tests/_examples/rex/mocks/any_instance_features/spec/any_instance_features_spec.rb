class Invoice
  def initialize(number)
    @number = number
  end

  attr_reader :number

  def total
    100
  end
end

describe "any instance features" do
  it "hands the receiving instance to the implementation block" do
    allow_any_instance_of(Invoice).to receive(:total) { |invoice| invoice.number * 10 }
    expect(Invoice.new(3).total).to eq(30)
  end

  it "stubs a message chain on any instance" do
    allow_any_instance_of(Invoice).to receive_message_chain(:customer, :name).and_return("Acme")
    expect(Invoice.new(1).customer.name).to eq("Acme")
  end

  it "fulfills an expectation with the instance that received it" do
    expect_any_instance_of(Invoice).to receive(:total).and_return(5)
    expect(Invoice.new(1).total).to eq(5)
  end

  it "restores the method after the example" do
    expect(Invoice.new(1).total).to eq(100)
  end
end
