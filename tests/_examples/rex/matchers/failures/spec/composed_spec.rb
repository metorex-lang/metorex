Person = Struct.new(:name, :age)

describe "composed and custom matchers" do
  it("satisfy") { expect(3).to satisfy { |value| value > 5 } }
  it("satisfy with a description") { expect(3).to satisfy("be large") { |value| value > 5 } }
  it("have_attributes") { expect(Person.new("Ada", 36)).to have_attributes(name: "Ada", age: 37) }
  it("and") { expect(5).to be_odd.and be > 6 }
  it("or") { expect(4).to be_odd.or be > 6 }
  it("include with a composed matcher") { expect(["apple"]).to include(a_string_matching(/pear/)) }
  it("operator ==") { expect(4).to be == 5 }
  it("a message string") { expect(4).to eq(5), "the total was off" }
  it("a message proc") { expect(4).to eq(5), -> { "computed message" } }
  it("aggregate_failures") do
    aggregate_failures "the totals" do
      expect(1).to eq(2)
      expect(3).to eq(4)
    end
  end
end

RSpec::Matchers.define :be_a_multiple_of do |expected|
  match { |actual| (actual % expected).zero? }
end

RSpec::Matchers.define :be_a_word do
  match { |actual| actual.match?(/\A\w+\z/) }
  failure_message { |actual| "expected #{actual.inspect} to be one word" }
  failure_message_when_negated { |actual| "expected #{actual.inspect} not to be one word" }
  description { "be a single word" }
end

describe "defined matchers" do
  it("default messages") { expect(10).to be_a_multiple_of(3) }
  it("default negated") { expect(9).not_to be_a_multiple_of(3) }
  it("custom messages") { expect("two words").to be_a_word }
  it("custom negated") { expect("word").not_to be_a_word }
  it { expect(9).to be_a_multiple_of(3) }
  it { expect("word").to be_a_word }
end
