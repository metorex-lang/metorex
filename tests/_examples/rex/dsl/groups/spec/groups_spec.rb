module Billing
  class Invoice
    def initialize(total = 0) = @total = total
    attr_reader :total
    def paid? = false
  end
end

describe "a string description" do
  it "runs" do
    expect(1).to eq(1)
  end
end

RSpec.describe Billing::Invoice do
  it "names the described class" do
    expect(described_class).to eq(Billing::Invoice)
  end

  it "builds an implicit subject from the described class" do
    expect(subject.total).to eq(0)
  end

  describe "#total" do
    it "keeps the described class in a nested group" do
      expect(described_class).to eq(Billing::Invoice)
    end

    it "reads the full description" do |example|
      expect(example.full_description).to eq("Billing::Invoice#total reads the full description")
    end

    context "when nested again" do
      specify "still knows its class" do
        expect(described_class).to eq(Billing::Invoice)
      end

      example "runs through example" do
        expect(self).to be_a(RSpec::Core::ExampleGroup)
      end
    end
  end

  it { is_expected.not_to be_paid }
  it { expect(subject.total).to eq(0) }
  it { expect(subject.total).to be > -1 }
end

RSpec.describe Billing do
  it "describes a module" do
    expect(described_class).to eq(Billing)
  end
end

describe "pending and skipped" do
  it "is pending with a reason", pending: "waiting on refunds" do
    expect(1).to eq(2)
  end

  it "calls pending in the body" do
    pending "rates are not final"
    expect(0.1 + 0.2).to eq(0.3)
  end

  it "passes while pending" do
    pending "this one works"
    expect(1).to eq(1)
  end

  it "calls skip in the body" do
    skip "needs a network"
    raise "never reached"
  end

  it "skips through metadata", skip: "flaky on CI" do
    raise "never reached"
  end

  xspecify "is turned off with xspecify" do
  end

  xexample "is turned off with xexample" do
  end

  it "is skipped with no reason", :skip do
  end
end

xdescribe "a group turned off with xdescribe" do
  it "does not run" do
    raise "never reached"
  end
end

xcontext "a group turned off with xcontext" do
  it "does not run either" do
    raise "never reached"
  end
end

fdescribe "a focused group" do
  fit "runs because nothing filters on focus" do
    expect(true).to be(true)
  end
end

context "a top-level context" do
  it "works like describe" do
    expect([1, 2]).to include(2)
  end
end
