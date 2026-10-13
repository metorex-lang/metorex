module RescueSpecs
  def self.load_once(attempts)
    attempts << :tried
    raise ArgumentError, "first try" if attempts.size < 2

    attempts
  rescue ArgumentError
    retry
  end

  def self.give_up(attempts)
    attempts << :tried
    raise ArgumentError, "always"
  rescue ArgumentError
    attempts.size < 2 ? retry : raise
  end

  def self.retry_with_if(attempts)
    attempts << :tried
    raise ArgumentError, "first try" if attempts.size < 2

    attempts
  rescue ArgumentError
    retry if attempts.size < 2
  end

  def self.reraise_plain
    raise "original"
  rescue
    raise ArgumentError, "replacement"
  end

  def self.reraise_without_cause
    raise "original"
  rescue => error
    replacement = NameError.new("replacement", :name)
    replacement.set_backtrace(error.backtrace)
    raise replacement, cause: nil
  end
end

RSpec.describe "retry in a method's rescue clause" do
  it "runs the method body again" do
    expect(RescueSpecs.load_once([])).to eq([:tried, :tried])
  end

  it "raises again once a condition stops retrying" do
    expect { RescueSpecs.give_up([]) }.to raise_error(ArgumentError, "always")
  end

  it "retries under a modifier if" do
    expect(RescueSpecs.retry_with_if([])).to eq([:tried, :tried])
  end
end

RSpec.describe "raise in a rescue clause" do
  it "records the exception being handled as the cause" do
    expect { RescueSpecs.reraise_plain }.to raise_error(ArgumentError) { |error| expect(error.cause.message).to eq("original") }
  end

  it "records no cause when given cause: nil" do
    expect { RescueSpecs.reraise_without_cause }.to raise_error(NameError) { |error| expect(error.cause).to be_nil }
  end
end
