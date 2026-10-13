class Mailer
  def deliver(message, to:) = "delivered #{message} to #{to}"
  def self.default_sender = "noreply@example.com"
end

describe "doubles" do
  it "rejects a message a strict double was not told about" do
    mailer = double("mailer")
    mailer.deliver
  end

  it "rejects a message on an anonymous double" do
    double.deliver("hello")
  end

  it "reports an expected message that never arrived" do
    mailer = double("mailer")
    expect(mailer).to receive(:deliver)
  end

  it "reports a message received too many times" do
    mailer = double("mailer")
    expect(mailer).to receive(:deliver).once
    mailer.deliver
    mailer.deliver
  end

  it "reports a message received with other arguments" do
    mailer = double("mailer")
    expect(mailer).to receive(:deliver).with("hello", to: "ada@example.com")
    mailer.deliver("goodbye", to: "ada@example.com")
  end

  it "reports a count that falls short" do
    mailer = double("mailer")
    expect(mailer).to receive(:deliver).exactly(3).times
    mailer.deliver
  end

  it "reports a message that was never to arrive" do
    mailer = double("mailer")
    expect(mailer).not_to receive(:deliver)
    mailer.deliver
  end

  it "reports at_least" do
    mailer = double("mailer")
    expect(mailer).to receive(:deliver).at_least(:twice)
    mailer.deliver
  end

  it "reports a stubbed double given unexpected arguments" do
    mailer = double("mailer")
    allow(mailer).to receive(:deliver).with(1).and_return(:ok)
    mailer.deliver(2)
  end
end

describe "partial doubles" do
  it "reports an expected message on a real object" do
    expect(Mailer).to receive(:default_sender)
  end
end

describe "spies" do
  it "reports a spy that did not receive the message" do
    mailer = spy("mailer")
    expect(mailer).to have_received(:deliver)
  end

  it "reports a spy given other arguments" do
    mailer = spy("mailer")
    mailer.deliver("hello")
    expect(mailer).to have_received(:deliver).with("goodbye")
  end
end

describe "verifying doubles" do
  it "rejects a method the class does not have" do
    instance_double(Mailer, archive: true)
  end

  it "rejects the wrong number of arguments" do
    mailer = instance_double(Mailer)
    allow(mailer).to receive(:deliver)
    mailer.deliver
  end

  it "rejects a class method the class does not have" do
    class_double(Mailer, archive_all: true)
  end

  it "rejects a missing keyword" do
    mailer = instance_double(Mailer)
    allow(mailer).to receive(:deliver)
    mailer.deliver("hello")
  end
end
