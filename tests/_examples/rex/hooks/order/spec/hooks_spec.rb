describe "hook order" do
  before(:all) do
    $events << "outer before all"
    @shared = "set in before all"
  end
  after(:all) { $events << "outer after all" }
  before { $events << "outer before" }
  after { $events << "outer after" }
  around(:example) do |example|
    $events << "outer around start"
    example.call
    $events << "outer around end"
  end

  it "runs outer hooks" do
    $events << "outer example"
    expect(@shared).to eq("set in before all")
  end

  context "nested" do
    before(:context) { $events << "inner before context" }
    after(:context) { $events << "inner after context" }
    before(:each) { $events << "inner before" }
    after(:example) { $events << "inner after" }
    after { $events << "inner after added later" }

    it "runs hooks from the outside in and the inside out" do
      $events << "inner example"
      expect(@shared).to eq("set in before all")
    end
  end
end

describe "the recorded order" do
  it "matches RSpec's" do
    expect($events).to eq([
      "suite before",
      "config before context", "outer before all",
      "config around start", "outer around start",
      "config before each", "outer before", "outer example",
      "outer after", "config after each",
      "outer around end", "config around end",
      "inner before context",
      "config around start", "outer around start",
      "config before each", "outer before", "inner before", "inner example",
      "inner after added later", "inner after", "outer after", "config after each",
      "outer around end", "config around end",
      "inner after context",
      "outer after all", "config after context",
      "config before context", "config around start", "config before each"
    ])
  end
end

describe "memoized helpers" do
  let(:calls) { [] }
  let(:account) { calls << :built; { balance: 100 } }
  let!(:created) { $events_created = true }

  it "builds a let once per example" do
    account
    account
    expect(calls).to eq([:built])
  end

  it "runs let! before the example" do
    expect($events_created).to be(true)
  end

  context "with a redefined let" do
    let(:account) { super().merge(balance: 250) }

    it "reaches the outer definition through super" do
      expect(account).to eq(balance: 250)
    end
  end

  describe Array do
    it "builds an implicit subject" do
      expect(subject).to eq([])
    end

    it { is_expected.to be_empty }
    it { should be_empty }

    context "with a named subject" do
      subject(:list) { [3, 1, 2] }

      it "reads the subject by name" do
        expect(list.sort).to eq([1, 2, 3])
      end

      it "reads the same object through subject" do
        expect(subject).to equal(list)
      end

      it { is_expected.to include(3) }
    end
  end

  describe "a string subject" do
    it { is_expected.to eq("a string subject") }
  end
end

describe "helper modules" do
  it "includes a configured module" do
    expect(money(1999)).to eq("$19.99")
  end

  context "with metadata", :counted do
    it "extends groups that match" do
      expect(self.class.group_count).to eq(3)
    end
  end

  context "without matching metadata" do
    it "leaves other groups alone" do
      expect(self.class).not_to respond_to(:group_count)
    end
  end

  context "when loud", loud: true do
    it "includes a module by metadata" do
      expect(shout("ready")).to eq("READY!")
    end
  end

  context "when prepended", :greeting do
    it "prepends a configured module" do
      expect(greet).to eq("hello from a group")
    end
  end
end
