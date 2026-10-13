class Gateway
  def charge(amount, currency: "USD") = "charged #{amount} #{currency}"
  def refund(amount) = "refunded #{amount}"
  def each_receipt = yield("receipt one")
  def self.connect = new
end

DEFAULT_TIMEOUT = 30

describe "doubles that pass" do
  it "answers stubbed messages" do
    gateway = double("gateway", charge: "ok", refund: "back")
    expect(gateway.charge).to eq("ok")
    expect(gateway.refund).to eq("back")
  end

  it "returns several values in order" do
    gateway = double("gateway")
    allow(gateway).to receive(:status).and_return(:pending, :settled)
    expect([gateway.status, gateway.status, gateway.status]).to eq(%i[pending settled settled])
  end

  it "raises, throws, and yields" do
    gateway = double("gateway")
    allow(gateway).to receive(:charge).and_raise(ArgumentError, "declined")
    allow(gateway).to receive(:halt).and_throw(:stop)
    allow(gateway).to receive(:each_receipt).and_yield("first").and_yield("second")
    expect { gateway.charge }.to raise_error(ArgumentError, "declined")
    expect { gateway.halt }.to throw_symbol(:stop)
    receipts = []
    gateway.each_receipt { |receipt| receipts << receipt }
    expect(receipts).to eq(%w[first second])
  end

  it "uses a block implementation" do
    gateway = double("gateway")
    allow(gateway).to receive(:charge) { |amount| amount * 2 }
    expect(gateway.charge(21)).to eq(42)
  end

  it "counts messages" do
    gateway = double("gateway")
    expect(gateway).to receive(:charge).twice
    expect(gateway).to receive(:refund).at_least(:once)
    expect(gateway).to receive(:void).never
    expect(gateway).to receive(:audit).exactly(3).times
    2.times { gateway.charge }
    gateway.refund
    3.times { gateway.audit }
  end

  it "matches arguments" do
    gateway = double("gateway")
    expect(gateway).to receive(:charge).with(100, currency: "EUR")
    expect(gateway).to receive(:record).with(anything, instance_of(Integer), kind_of(Numeric))
    expect(gateway).to receive(:log).with(hash_including(level: :info))
    expect(gateway).to receive(:batch).with(array_including(2))
    expect(gateway).to receive(:ping).with(no_args)
    expect(gateway).to receive(:trace).with(any_args)
    expect(gateway).to receive(:label).with(a_string_starting_with("inv"))
    gateway.charge(100, currency: "EUR")
    gateway.record(:x, 1, 2.5)
    gateway.log(level: :info, message: "ok")
    gateway.batch([1, 2, 3])
    gateway.ping
    gateway.trace(1, 2, 3)
    gateway.label("invoice")
  end

  it "stubs several messages at once" do
    gateway = double("gateway")
    allow(gateway).to receive_messages(charge: "ok", refund: "back")
    expect([gateway.charge, gateway.refund]).to eq(%w[ok back])
  end

  it "stubs a chain of messages" do
    gateway = double("gateway")
    allow(gateway).to receive_message_chain(:account, :owner, :name).and_return("Ada")
    expect(gateway.account.owner.name).to eq("Ada")
  end

  it "records messages on a spy" do
    gateway = spy("gateway")
    gateway.charge(5)
    gateway.charge(5)
    expect(gateway).to have_received(:charge).with(5).twice
    expect(gateway).not_to have_received(:refund)
  end

  it "records messages on a stubbed partial double" do
    gateway = Gateway.new
    allow(gateway).to receive(:refund)
    gateway.refund(3)
    expect(gateway).to have_received(:refund).with(3)
  end
end

describe "partial doubles that pass" do
  it "stubs a class method for one example" do
    allow(Gateway).to receive(:connect).and_return(:fake)
    expect(Gateway.connect).to eq(:fake)
  end

  it "has the original class method back in the next example" do
    expect(Gateway.connect).to be_a(Gateway)
  end

  it "calls through to the original" do
    gateway = Gateway.new
    expect(gateway).to receive(:charge).and_call_original
    expect(gateway.charge(10)).to eq("charged 10 USD")
  end

  it "wraps the original" do
    gateway = Gateway.new
    allow(gateway).to receive(:refund).and_wrap_original { |original, amount| original.call(amount * 2) }
    expect(gateway.refund(4)).to eq("refunded 8")
  end

  it "stubs any instance" do
    allow_any_instance_of(Gateway).to receive(:refund).and_return("stubbed")
    expect(Gateway.new.refund(1)).to eq("stubbed")
  end

  it "has the original instance method back in the next example" do
    expect(Gateway.new.refund(1)).to eq("refunded 1")
  end

  it "expects a message on any instance" do
    expect_any_instance_of(Gateway).to receive(:charge).with(7).and_return("seen")
    expect(Gateway.new.charge(7)).to eq("seen")
  end

  it "stubs a constant" do
    stub_const("DEFAULT_TIMEOUT", 5)
    expect(DEFAULT_TIMEOUT).to eq(5)
  end

  it "has the original constant back in the next example" do
    expect(DEFAULT_TIMEOUT).to eq(30)
  end
end

describe "verifying doubles that pass" do
  it "answers a method the class has" do
    gateway = instance_double(Gateway, refund: "back")
    expect(gateway.refund(1)).to eq("back")
  end

  it "checks keywords against the real signature" do
    gateway = instance_double(Gateway)
    allow(gateway).to receive(:charge)
    gateway.charge(5, currency: "EUR")
    expect(gateway).to have_received(:charge).with(5, currency: "EUR")
  end

  it "doubles a class" do
    gateway_class = class_double(Gateway, connect: :connected)
    expect(gateway_class.connect).to eq(:connected)
  end

  it "doubles an object" do
    gateway = object_double(Gateway.new, refund: "back")
    expect(gateway.refund(2)).to eq("back")
  end

  it "makes an instance spy" do
    gateway = instance_spy(Gateway)
    gateway.refund(9)
    expect(gateway).to have_received(:refund).with(9)
  end
end
